//! Authentication and authorization

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

use nanokvm_core::{Config, Error, Result};

/// Account file path
const ACCOUNT_FILE: &str = "/etc/kvm/account";
const PASSWORD_FILE: &str = "/etc/kvm/pwd";

/// Token expiration (2 hours)
const TOKEN_EXPIRATION_SECS: u64 = 7200;
/// Refresh token expiration (7 days)
const REFRESH_TOKEN_EXPIRATION_SECS: u64 = 604800;

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (username)
    pub sub: String,
    /// Expiration timestamp
    pub exp: u64,
    /// Issued at timestamp
    pub iat: u64,
    /// JWT ID
    pub jti: String,
    /// Token type
    pub token_type: TokenType,
}

/// Token type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    Access,
    Refresh,
}

/// Authentication response
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

/// User account
#[derive(Debug, Clone)]
pub struct Account {
    pub username: String,
    pub password_hash: String,
}

/// Failed login tracking
struct FailedLogin {
    count: u32,
    first_failure: SystemTime,
}

/// Authentication manager
pub struct AuthManager {
    secret_key: String,
    accounts: RwLock<HashMap<String, Account>>,
    failed_logins: RwLock<HashMap<String, FailedLogin>>,
    revoked_tokens: RwLock<Vec<String>>,
}

impl AuthManager {
    /// Create a new authentication manager
    pub fn new() -> Self {
        let config = Config::instance().read();
        let secret_key = config.jwt.secret_key.clone();
        drop(config);

        let mut manager = Self {
            secret_key,
            accounts: RwLock::new(HashMap::new()),
            failed_logins: RwLock::new(HashMap::new()),
            revoked_tokens: RwLock::new(Vec::new()),
        };

        // Load accounts
        if let Err(e) = manager.load_accounts() {
            warn!("Failed to load accounts: {}", e);
        }

        manager
    }

    /// Load accounts from files
    fn load_accounts(&mut self) -> Result<()> {
        self.load_accounts_from(Path::new(ACCOUNT_FILE), Path::new(PASSWORD_FILE))
    }

    fn load_accounts_from(&mut self, account_file: &Path, password_file: &Path) -> Result<()> {
        let mut accounts = self.accounts.write();

        // Existing but invalid account data must never trigger default credentials.
        match fs::read_to_string(account_file) {
            Ok(content) => {
                *accounts = parse_accounts(&content)?;
                return Ok(());
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        match fs::read_to_string(password_file) {
            Ok(content) => {
                let password = legacy_plaintext_password(&content)?;
                let hash = self.hash_password(password)?;
                accounts.insert("admin".to_string(), Account {
                    username: "admin".to_string(), password_hash: hash,
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        // Create default account if none exist
        if accounts.is_empty() {
            let hash = self.hash_password("admin")?;
            accounts.insert(
                "admin".to_string(),
                Account {
                    username: "admin".to_string(),
                    password_hash: hash,
                },
            );
            info!("Created default admin account");
        }

        Ok(())
    }

    /// Hash a password using Argon2
    fn hash_password(&self, password: &str) -> Result<String> {
        let mut salt_bytes = [0u8; 16];
        rand::fill(&mut salt_bytes);
        let salt = SaltString::encode_b64(&salt_bytes)
            .map_err(|e| Error::auth(format!("Salt generation failed: {}", e)))?;
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| Error::auth(format!("Password hashing failed: {}", e)))
    }

    /// Verify a password against a hash
    fn verify_password(&self, password: &str, hash: &str) -> bool {
        let Ok(parsed_hash) = PasswordHash::new(hash) else {
            return false;
        };

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }

    /// Authenticate a user
    pub fn login(&self, username: &str, password: &str) -> Result<AuthResponse> {
        // Check for lockout
        if self.is_locked_out(username) {
            return Err(Error::RateLimitExceeded);
        }

        // Find account
        let accounts = self.accounts.read();
        let account = accounts
            .get(username)
            .ok_or_else(|| Error::auth("Invalid credentials"))?;

        // Verify password
        if !self.verify_password(password, &account.password_hash) {
            drop(accounts);
            self.record_failed_login(username);
            return Err(Error::auth("Invalid credentials"));
        }

        drop(accounts);

        // Clear failed login attempts
        self.clear_failed_logins(username);

        // Generate tokens
        let access_token = self.generate_token(username, TokenType::Access)?;
        let refresh_token = self.generate_token(username, TokenType::Refresh)?;

        info!("User '{}' logged in successfully", username);

        Ok(AuthResponse {
            access_token,
            refresh_token,
            expires_in: TOKEN_EXPIRATION_SECS,
        })
    }

    /// Generate a JWT token
    fn generate_token(&self, username: &str, token_type: TokenType) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expiration = match token_type {
            TokenType::Access => now + TOKEN_EXPIRATION_SECS,
            TokenType::Refresh => now + REFRESH_TOKEN_EXPIRATION_SECS,
        };

        let claims = Claims {
            sub: username.to_string(),
            exp: expiration,
            iat: now,
            jti: Uuid::new_v4().to_string(),
            token_type,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret_key.as_bytes()),
        )
        .map_err(|e| Error::auth(format!("Token generation failed: {}", e)))
    }

    /// Validate a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        // Check if token is revoked
        if self.is_token_revoked(token) {
            return Err(Error::unauthorized("Token revoked"));
        }

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret_key.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| Error::unauthorized(format!("Invalid token: {}", e)))?;

        Ok(token_data.claims)
    }

    /// Validate a token for access to protected API routes.
    pub fn validate_access_token(&self, token: &str) -> Result<Claims> {
        let claims = self.validate_token(token)?;
        if claims.token_type != TokenType::Access {
            return Err(Error::unauthorized("Access token required"));
        }
        Ok(claims)
    }

    /// Refresh an access token
    pub fn refresh_token(&self, refresh_token: &str) -> Result<AuthResponse> {
        let claims = self.validate_token(refresh_token)?;

        if claims.token_type != TokenType::Refresh {
            return Err(Error::auth("Invalid token type"));
        }

        let access_token = self.generate_token(&claims.sub, TokenType::Access)?;
        let new_refresh_token = self.generate_token(&claims.sub, TokenType::Refresh)?;

        Ok(AuthResponse {
            access_token,
            refresh_token: new_refresh_token,
            expires_in: TOKEN_EXPIRATION_SECS,
        })
    }

    /// Logout and optionally revoke tokens
    pub fn logout(&self, token: &str) {
        let config = Config::instance().read();
        if config.jwt.revoke_tokens_on_logout {
            self.revoke_token(token);
        }
    }

    /// Revoke a token
    fn revoke_token(&self, token: &str) {
        self.revoked_tokens.write().push(token.to_string());
    }

    /// Check if a token is revoked
    fn is_token_revoked(&self, token: &str) -> bool {
        self.revoked_tokens.read().contains(&token.to_string())
    }

    /// Record a failed login attempt
    fn record_failed_login(&self, username: &str) {
        let mut failed = self.failed_logins.write();
        let entry = failed.entry(username.to_string()).or_insert(FailedLogin {
            count: 0,
            first_failure: SystemTime::now(),
        });
        entry.count += 1;
        debug!(
            "Failed login attempt {} for user '{}'",
            entry.count, username
        );
    }

    /// Clear failed login attempts
    fn clear_failed_logins(&self, username: &str) {
        self.failed_logins.write().remove(username);
    }

    /// Check if a user is locked out
    fn is_locked_out(&self, username: &str) -> bool {
        let config = Config::instance().read();
        let max_failures = config.security.login_max_failures as u32;
        let lockout_duration = Duration::from_secs(config.security.login_lockout_duration as u64);
        drop(config);

        let failed = self.failed_logins.read();
        if let Some(entry) = failed.get(username)
            && entry.count >= max_failures
            && entry.first_failure.elapsed().unwrap_or_default() < lockout_duration
        {
            return true;
        }
        false
    }

    pub fn has_account(&self, username: &str) -> bool {
        self.accounts.read().contains_key(username)
    }

    pub fn password_updated(&self, username: &str) -> Result<bool> {
        let hash = self.accounts.read().get(username)
            .ok_or_else(|| Error::auth("Account unavailable"))?.password_hash.clone();
        let parsed = PasswordHash::new(&hash)
            .map_err(|_| Error::auth("Invalid account password hash"))?;
        match Argon2::default().verify_password(b"admin", &parsed) {
            Ok(()) => Ok(false),
            Err(argon2::password_hash::Error::Password) => Ok(true),
            Err(_) => Err(Error::auth("Unable to verify password status")),
        }
    }

    /// Change user password
    pub fn change_password(
        &self,
        username: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<()> {
        self.change_password_with_save(username, old_password, new_password, |accounts| {
            self.save_accounts(accounts)
        })
    }

    fn change_password_with_save(
        &self,
        username: &str,
        old_password: &str,
        new_password: &str,
        save: impl FnOnce(&HashMap<String, Account>) -> Result<()>,
    ) -> Result<()> {
        let mut accounts = self.accounts.write();

        let account = accounts
            .get(username)
            .ok_or_else(|| Error::auth("User not found"))?;

        if !self.verify_password(old_password, &account.password_hash) {
            return Err(Error::auth("Invalid current password"));
        }

        let new_hash = self.hash_password(new_password)?;
        let mut updated_accounts = accounts.clone();
        updated_accounts.insert(
            username.to_string(),
            Account {
                username: username.to_string(),
                password_hash: new_hash,
            },
        );

        // Publish the new account only after persistence succeeds.
        save(&updated_accounts)?;
        *accounts = updated_accounts;

        info!("Password changed for user '{}'", username);
        Ok(())
    }

    /// Save accounts to file
    fn save_accounts(&self, accounts: &HashMap<String, Account>) -> Result<()> {
        let content: String = accounts
            .values()
            .map(|acc| format!("{}:{}", acc.username, acc.password_hash))
            .collect::<Vec<_>>()
            .join("\n");

        fs::create_dir_all("/etc/kvm")?;
        write_account_file(Path::new(ACCOUNT_FILE), content.as_bytes())?;
        Ok(())
    }
}

fn parse_accounts(content: &str) -> Result<HashMap<String, Account>> {
    let mut accounts = HashMap::new();
    for line in content.lines() {
        let (username, hash) = line.split_once(':')
            .ok_or_else(|| Error::auth("Malformed account record"))?;
        if username.is_empty() || username.chars().any(char::is_control) {
            return Err(Error::auth("Invalid account username"));
        }
        let parsed = PasswordHash::new(hash)
            .map_err(|_| Error::auth("Invalid account password hash"))?;
        if !matches!(parsed.algorithm.as_str(), "argon2id" | "argon2i" | "argon2d")
            || parsed.salt.is_none() || parsed.hash.is_none() {
            return Err(Error::auth("Unsupported account password hash"));
        }
        if accounts.insert(username.to_string(), Account {
            username: username.to_string(), password_hash: hash.to_string(),
        }).is_some() {
            return Err(Error::auth("Duplicate account username"));
        }
    }
    if accounts.is_empty() {
        return Err(Error::auth("Empty account file"));
    }
    Ok(accounts)
}

fn legacy_plaintext_password(content: &str) -> Result<&str> {
    let password = content.trim();
    if password.is_empty() || password.starts_with('{') || password.starts_with('[') {
        return Err(Error::auth("Legacy account format requires explicit migration"));
    }
    Ok(password)
}

// Write beside the destination, then rename: failed writes never truncate the live file.
fn write_account_file(path: &Path, content: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| std::io::Error::new(
        std::io::ErrorKind::InvalidInput, "account file needs a parent directory",
    ))?;
    let temporary = parent.join(format!(".account-{}.tmp", Uuid::new_v4()));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary)?;
    let result = (|| {
        file.write_all(content)?;
        file.sync_all()?;
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    // Rename is the commit point. A directory-sync failure must not report a
    // rejected password change after the new file has already become visible.
    if result.is_ok() {
        if let Err(error) = fs::File::open(parent).and_then(|directory| directory.sync_all()) {
            warn!("Account file replaced, but directory sync failed: {}", error);
        }
    }
    result
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub(crate) mod access_tests {
    use super::*;
    pub(crate) fn manager() -> AuthManager {
        AuthManager {
            secret_key: "synthetic-test-key-not-a-real-credential".into(),
            accounts: RwLock::new(HashMap::new()),
            failed_logins: RwLock::new(HashMap::new()),
            revoked_tokens: RwLock::new(Vec::new()),
        }
    }
    #[test]
    fn access_token_is_accepted() {
        let manager = manager();
        let token = manager.generate_token("test", TokenType::Access).unwrap();
        assert!(manager.validate_access_token(&token).is_ok());
    }
    #[test]
    fn refresh_token_cannot_authorize_api_access() {
        let manager = manager();
        let token = manager.generate_token("test", TokenType::Refresh).unwrap();
        assert!(manager.validate_access_token(&token).is_err());
        assert!(manager.refresh_token(&token).is_ok());
    }
    #[test]
    fn revoked_and_invalid_tokens_are_rejected() {
        let manager = manager();
        let token = manager.generate_token("test", TokenType::Access).unwrap();
        manager.revoke_token(&token);
        assert!(manager.validate_access_token(&token).is_err());
        assert!(manager.validate_access_token("invalid").is_err());
    }
    pub(crate) fn password_manager() -> AuthManager {
        manager_with_password("synthetic-old")
    }
    pub(crate) fn manager_with_password(password: &str) -> AuthManager {
        let manager = manager();
        let hash = manager.hash_password(password).unwrap();
        manager.accounts.write().insert("test".into(), Account {
            username: "test".into(), password_hash: hash,
        });
        manager
    }

    #[test]
    fn failed_password_save_preserves_live_account() {
        let manager = password_manager();
        let before = manager.accounts.read()["test"].password_hash.clone();
        let result = manager.change_password_with_save("test", "synthetic-old", "synthetic-new", |_| {
            Err(Error::auth("synthetic persistence failure"))
        });
        assert!(result.is_err());
        assert_eq!(manager.accounts.read()["test"].password_hash, before);
    }

    #[test]
    fn successful_password_save_publishes_saved_hash() {
        let manager = password_manager();
        let mut saved_hash = String::new();
        manager.change_password_with_save("test", "synthetic-old", "synthetic-new", |accounts| {
            saved_hash = accounts["test"].password_hash.clone();
            Ok(())
        }).unwrap();
        assert_eq!(manager.accounts.read()["test"].password_hash, saved_hash);
        assert!(manager.verify_password("synthetic-new", &saved_hash));
        assert!(!manager.verify_password("synthetic-old", &saved_hash));
    }

    #[test]
    fn wrong_current_password_never_calls_storage() {
        let manager = password_manager();
        let before = manager.accounts.read()["test"].password_hash.clone();
        assert!(manager.change_password_with_save("test", "wrong", "synthetic-new", |_| {
            panic!("storage must not be called for an invalid current password");
        }).is_err());
        assert_eq!(manager.accounts.read()["test"].password_hash, before);
    }

    struct TestDirectory(std::path::PathBuf);
    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("nanokvm-account-test-{}", Uuid::new_v4()));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestDirectory {
        fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
    }

    #[test]
    fn account_file_replacement_is_complete_and_private() {
        let dir = TestDirectory::new();
        let path = dir.0.join("account");
        fs::write(&path, "synthetic-old").unwrap();
        write_account_file(&path, b"synthetic-new").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"synthetic-new");
        assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        }
    }

    #[test]
    fn failed_account_rename_preserves_destination_and_cleans_temporary() {
        let dir = TestDirectory::new();
        let path = dir.0.join("account");
        fs::create_dir(&path).unwrap();
        fs::write(path.join("sentinel"), "untouched").unwrap();
        assert!(write_account_file(&path, b"synthetic-new").is_err());
        assert_eq!(fs::read_to_string(path.join("sentinel")).unwrap(), "untouched");
        assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn account_replacement_does_not_follow_destination_symlink() {
        let dir = TestDirectory::new();
        let target = dir.0.join("other");
        fs::write(&target, "untouched").unwrap();
        let path = dir.0.join("account");
        std::os::unix::fs::symlink(&target, &path).unwrap();
        write_account_file(&path, b"synthetic-new").unwrap();
        assert_eq!(fs::read_to_string(target).unwrap(), "untouched");
        assert!(!fs::symlink_metadata(&path).unwrap().file_type().is_symlink());
    }

    #[test]
    fn account_parser_accepts_valid_record_and_rejects_partial_or_duplicate_data() {
        let manager = manager();
        let hash = manager.hash_password("synthetic").unwrap();
        let record = format!("test:{hash}");
        assert_eq!(parse_accounts(&record).unwrap()["test"].password_hash, hash);
        for invalid in [String::new(), "junk".into(), "test:not-a-hash".into(),
            format!(":{hash}"), format!("{record}\nmalformed"), format!("{record}\n{record}")] {
            assert!(parse_accounts(&invalid).is_err());
        }
    }

    #[test]
    fn legacy_json_is_not_treated_as_plaintext_password() {
        assert!(legacy_plaintext_password(r#"{"username":"user","password":"synthetic-bcrypt"}"#).is_err());
        assert!(legacy_plaintext_password("   ").is_err());
        assert_eq!(legacy_plaintext_password("synthetic-old\n").unwrap(), "synthetic-old");
    }

    #[test]
    fn corrupt_account_file_never_falls_back_to_legacy_or_default() {
        let dir = TestDirectory::new();
        let account = dir.0.join("account");
        let legacy = dir.0.join("pwd");
        fs::write(&legacy, "synthetic-legacy").unwrap();
        for content in ["", "broken", "test:not-a-hash"] {
            fs::write(&account, content).unwrap();
            let mut auth = manager();
            assert!(auth.load_accounts_from(&account, &legacy).is_err());
            assert!(auth.accounts.read().is_empty());
        }
        fs::remove_file(&account).unwrap();
        fs::create_dir(&account).unwrap();
        let mut auth = manager();
        assert!(auth.load_accounts_from(&account, &legacy).is_err());
        assert!(auth.accounts.read().is_empty());
    }

    #[test]
    fn loader_preserves_legacy_json_and_does_not_create_default_account() {
        let dir = TestDirectory::new();
        let account = dir.0.join("account");
        let legacy = dir.0.join("pwd");
        let original = r#"{"username":"test","password":"synthetic-bcrypt"}"#;
        fs::write(&legacy, original).unwrap();
        let mut auth = manager();
        assert!(auth.load_accounts_from(&account, &legacy).is_err());
        assert!(auth.accounts.read().is_empty());
        assert_eq!(fs::read_to_string(&legacy).unwrap(), original);
        assert!(!account.exists());
    }

    #[test]
    fn loader_uses_valid_primary_before_legacy_and_supports_missing_files() {
        let dir = TestDirectory::new();
        let account = dir.0.join("account");
        let legacy = dir.0.join("pwd");
        let mut auth = manager();
        auth.load_accounts_from(&account, &legacy).unwrap();
        assert!(auth.verify_password("admin", &auth.accounts.read()["admin"].password_hash));
        let hash = auth.hash_password("synthetic-primary").unwrap();
        fs::write(&account, format!("test:{hash}")).unwrap();
        fs::write(&legacy, "{unsupported-json}").unwrap();
        let mut primary = manager();
        primary.load_accounts_from(&account, &legacy).unwrap();
        assert_eq!(primary.accounts.read().len(), 1);
        assert_eq!(primary.accounts.read()["test"].password_hash, hash);
        fs::remove_file(&account).unwrap();
        fs::write(&legacy, "synthetic-legacy").unwrap();
        let mut old = manager();
        old.load_accounts_from(&account, &legacy).unwrap();
        assert!(old.verify_password("synthetic-legacy", &old.accounts.read()["admin"].password_hash));
    }

}
