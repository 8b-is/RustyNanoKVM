//! Authentication and authorization

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use argon2::{
<<<<<<< Updated upstream
<<<<<<< Updated upstream
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
=======
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
=======
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
=======
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
>>>>>>> 1220bc0 (fix: Fix compilation errors and pass all tests)
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
use tracing::{debug, info, warn};
=======
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
<<<<<<< HEAD
use tracing::{debug, error, info, warn};
>>>>>>> febff1d (feat: Add Rust workspace structure with all core crates and infrastructure)
=======
use tracing::{debug, info, warn};
>>>>>>> 1220bc0 (fix: Fix compilation errors and pass all tests)
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
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
        let mut accounts = self.accounts.write();

        // Try new account format first
        if Path::new(ACCOUNT_FILE).exists() {
            let content = fs::read_to_string(ACCOUNT_FILE)?;
            for line in content.lines() {
                if let Some((username, password_hash)) = line.split_once(':') {
                    accounts.insert(
                        username.to_string(),
                        Account {
                            username: username.to_string(),
                            password_hash: password_hash.to_string(),
                        },
                    );
                }
            }
        }

        // Fall back to legacy password file
        if accounts.is_empty() && Path::new(PASSWORD_FILE).exists() {
            let password = fs::read_to_string(PASSWORD_FILE)?.trim().to_string();
            let hash = self.hash_password(&password)?;
            accounts.insert(
                "admin".to_string(),
                Account {
                    username: "admin".to_string(),
                    password_hash: hash,
                },
            );
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
<<<<<<< Updated upstream
=======
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> 1220bc0 (fix: Fix compilation errors and pass all tests)
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
        // Generate a random salt using UUID
        let salt_string = Uuid::new_v4().to_string().replace("-", "");
        let salt = SaltString::from_b64(&salt_string[..22])
            .map_err(|e| Error::auth(format!("Salt generation failed: {}", e)))?;
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
        
=======
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
<<<<<<< HEAD

=======
        let salt = SaltString::generate(&mut OsRng);
>>>>>>> febff1d (feat: Add Rust workspace structure with all core crates and infrastructure)
=======
        
>>>>>>> 1220bc0 (fix: Fix compilation errors and pass all tests)
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======

>>>>>>> Stashed changes
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

    /// Change user password
    pub fn change_password(
        &self,
        username: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<()> {
        let mut accounts = self.accounts.write();

        let account = accounts
            .get(username)
            .ok_or_else(|| Error::auth("User not found"))?;

        if !self.verify_password(old_password, &account.password_hash) {
            return Err(Error::auth("Invalid current password"));
        }

        let new_hash = self.hash_password(new_password)?;
        accounts.insert(
            username.to_string(),
            Account {
                username: username.to_string(),
                password_hash: new_hash,
            },
        );

        // Save to file
        self.save_accounts(&accounts)?;

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
        fs::write(ACCOUNT_FILE, content)?;
        Ok(())
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}
