//! Middleware for authentication, rate limiting, etc.

use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode, Uri},
    middleware::Next,
    response::Response,
};
use tracing::debug;

use crate::state::AppState;

/// Identity established by access-token verification, never by request parameters.
#[derive(Clone)]
pub struct AuthenticatedUser(pub String);

/// Authenticate ordinary API requests using an access token.
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    authenticate(state, request, next, false).await
}

/// Browsers cannot attach an Authorization header to WebSocket handshakes.
/// Accept their existing token cookie only on a same-origin upgrade request.
pub async fn websocket_auth_middleware(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    authenticate(state, request, next, true).await
}

async fn authenticate(
    state: Arc<AppState>,
    mut request: Request<Body>,
    next: Next,
    websocket: bool,
) -> Result<Response, StatusCode> {
    // Release the non-Send configuration guard before awaiting the handler.
    let (auth_disabled, scheme) = {
        let config = state.config();
        (config.is_auth_disabled(), config.proto.clone())
    };
    if auth_disabled {
        return Ok(next.run(request).await);
    }

    let headers = request.headers();
    // An explicit but invalid Authorization header must never fall back to a cookie.
    let token = if headers.contains_key("authorization") {
        single_header(headers, "authorization").and_then(|v| v.strip_prefix("Bearer "))
    } else if websocket {
        websocket_cookie(headers, &scheme)
    } else {
        None
    };
    let token = token.ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = state
        .auth
        .validate_access_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    request
        .extensions_mut()
        .insert(AuthenticatedUser(claims.sub));
    debug!("Token validated successfully");
    Ok(next.run(request).await)
}

fn single_header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    let mut values = headers.get_all(name).iter();
    let value = values.next()?.to_str().ok()?;
    if values.next().is_some() {
        return None;
    }
    Some(value)
}

fn websocket_cookie<'a>(headers: &'a HeaderMap, scheme: &str) -> Option<&'a str> {
    if !single_header(headers, "upgrade")?.eq_ignore_ascii_case("websocket") {
        return None;
    }
    // Use configured protocol and direct Host, never untrusted forwarding headers.
    // TLS-terminating proxies must preserve Host and configure the external protocol.
    if !matches!(scheme, "http" | "https") {
        return None;
    }
    let origin: Uri = single_header(headers, "origin")?.parse().ok()?;
    let authority = origin.authority()?.as_str();
    if origin.scheme_str()? != scheme
        || !authority.eq_ignore_ascii_case(single_header(headers, "host")?)
        || authority.contains('@')
        || origin.query().is_some()
        || !matches!(origin.path(), "" | "/")
    {
        return None;
    }
    let mut token = None;
    for cookie in headers.get_all("cookie").iter() {
        for pair in cookie.to_str().ok()?.split(';') {
            let (name, value) = pair.trim().split_once('=')?;
            if name == "nano-kvm-token" {
                if token.is_some() || value.is_empty() {
                    return None;
                }
                token = Some(value);
            }
        }
    }
    token
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (name, value) in [
            ("upgrade", "websocket"),
            ("host", "localhost:8080"),
            ("origin", "http://localhost:8080"),
            ("cookie", "nano-kvm-token=synthetic"),
        ] {
            headers.insert(name, value.parse().unwrap());
        }
        headers
    }

    #[test]
    fn rejects_ambiguous_credentials_and_origins() {
        let valid = headers();
        assert_eq!(websocket_cookie(&valid, "http"), Some("synthetic"));
        for name in ["origin", "host", "upgrade", "cookie"] {
            let mut duplicate = valid.clone();
            duplicate.append(name, valid[name].clone());
            assert_eq!(websocket_cookie(&duplicate, "http"), None, "{name}");
        }
        for origin in [
            "null",
            "http://foreign:8080",
            "https://localhost:8080",
            "http://localhost:8080/path",
            "http://localhost:8080?query",
            "http://user@localhost:8080",
        ] {
            let mut invalid = valid.clone();
            invalid.insert("origin", origin.parse().unwrap());
            assert_eq!(websocket_cookie(&invalid, "http"), None, "{origin}");
        }
        for cookie in [
            "nano-kvm-token=",
            "nano-kvm-token=one; nano-kvm-token=two",
            "other=one",
            "malformed",
        ] {
            let mut invalid = valid.clone();
            invalid.insert("cookie", cookie.parse().unwrap());
            assert_eq!(websocket_cookie(&invalid, "http"), None, "{cookie}");
        }
    }
}
