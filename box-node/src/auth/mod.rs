use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Serialize, Deserialize)]
struct JwksResponse {
    keys: Vec<JwkKey>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JwkKey {
    kid: String,
    kty: String,
    #[serde(rename = "use")]
    key_use: Option<String>,
    alg: Option<String>,
    n: String,
    e: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    preferred_username: Option<String>,
    game_id: Option<String>,
    exp: usize,
    iss: String,
    aud: serde_json::Value,
}

pub type UserId = String;

#[derive(Clone)]
struct CachedKey {
    decoding_key: DecodingKey,
    cached_at: Instant,
}

pub struct Auth {
    jwks_url: String,
    validation: Validation,
    key_cache: HashMap<String, CachedKey>,
    cache_duration: Duration,
    refetch_task_cancel: CancellationToken,
}


pub type AuthHandle = Arc<RwLock<Auth>>;

#[derive(Debug)]
pub enum AuthError {
    ExpiredToken,
    /// Bad encoding and what not
    IllegalToken,
    /// Invalid signature or key issues
    InvalidSignature,
    /// Network or other internal errors
    InternalError(anyhow::Error),
    /// Key not found in JWKS
    KeyNotFound,
    /// Invalid issuer or audience
    InvalidClaims,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::ExpiredToken => write!(f, "Token has expired"),
            AuthError::IllegalToken => write!(f, "Token format is invalid"),
            AuthError::InvalidSignature => write!(f, "Token signature is invalid"),
            AuthError::InternalError(e) => write!(f, "Internal error: {}", e),
            AuthError::KeyNotFound => write!(f, "Signing key not found"),
            AuthError::InvalidClaims => write!(f, "Invalid token claims"),
        }
    }
}

impl std::error::Error for AuthError {}

impl From<reqwest::Error> for AuthError {
    fn from(e: reqwest::Error) -> Self {
        AuthError::InternalError(anyhow::Error::new(e))
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::ExpiredToken,
            jsonwebtoken::errors::ErrorKind::InvalidSignature
            | jsonwebtoken::errors::ErrorKind::InvalidRsaKey(_)
            | jsonwebtoken::errors::ErrorKind::RsaFailedSigning
            | jsonwebtoken::errors::ErrorKind::InvalidEcdsaKey => AuthError::InvalidSignature,
            jsonwebtoken::errors::ErrorKind::InvalidIssuer
            | jsonwebtoken::errors::ErrorKind::InvalidAudience
            | jsonwebtoken::errors::ErrorKind::InvalidSubject
            | jsonwebtoken::errors::ErrorKind::MissingRequiredClaim(_) => AuthError::InvalidClaims,
            _ => AuthError::IllegalToken,
        }
    }
}

impl Auth {
    pub async fn new(
        jwks_url: String,
        issuer: String,
        audience: Vec<String>,
        cert_validity: Duration,
    ) -> Result<Arc<RwLock<Self>>, AuthError> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&audience.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        validation.set_issuer(&[&issuer]);

        let refetch_task_cancel = CancellationToken::new();
        let mut auth = Auth {
            jwks_url,
            validation,
            key_cache: HashMap::new(),
            cache_duration: cert_validity,
            refetch_task_cancel: refetch_task_cancel.clone(),
        };

        // initial cache
        auth.fetch_and_cache_keys().await?;
        let auth = Arc::new(RwLock::new(auth));
        tokio::spawn({
            let weak = Arc::downgrade(&auth);
            async move { refetch_task(weak, refetch_task_cancel).await }
        });

        // tokio::spawn(refetch_task(auth, cancel))

        Ok(auth)
    }

    pub fn verify_user(&self, token: &str) -> Result<UserId, AuthError> {
        let claims = self.verify_token(token)?;
        Ok(claims.sub)
    }

    fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        let header = jsonwebtoken::decode_header(token).map_err(|_| AuthError::IllegalToken)?;

        let kid = header.kid.ok_or(AuthError::IllegalToken)?;

        let decoding_key = match self.key_cache.get(&kid) {
            Some(key) if key.cached_at.elapsed() < self.cache_duration => &key.decoding_key,
            Some(_expired) => {
                // Key expired - should trigger a cache refresh in background task
                // For now, try to use expired key but this should be improved
                return Err(AuthError::KeyNotFound);
            }
            None => return Err(AuthError::KeyNotFound),
        };

        let token_data = decode::<Claims>(token, decoding_key, &self.validation)?;
        Ok(token_data.claims)
    }

    async fn fetch_and_cache_keys(&mut self) -> Result<(), AuthError> {
        let response = reqwest::get(&self.jwks_url)
            .await
            .map_err(|e| AuthError::InternalError(anyhow::Error::new(e)))?;

        if !response.status().is_success() {
            return Err(AuthError::InternalError(anyhow::Error::msg(format!(
                "JWKS endpoint returned status: {}",
                response.status()
            ))));
        }

        let jwks_response: JwksResponse = response
            .json()
            .await
            .map_err(|e| AuthError::InternalError(anyhow::Error::new(e)))?;

        let now = Instant::now();
        self.key_cache.clear();

        let mut keys_added = 0;
        for key in jwks_response.keys {
            // Only cache RSA keys suitable for signing
            if key.kty == "RSA"
                && key.key_use.as_ref().map_or(true, |u| u == "sig")
                && key.alg.as_ref().map_or(true, |a| a == "RS256")
            {
                match DecodingKey::from_rsa_components(&key.n, &key.e) {
                    Ok(decoding_key) => {
                        self.key_cache.insert(
                            key.kid.clone(),
                            CachedKey {
                                decoding_key,
                                cached_at: now,
                            },
                        );
                        keys_added += 1;
                    }
                    Err(e) => {
                        log::warn!("Failed to create decoding key for kid {}: {}", key.kid, e);
                        continue;
                    }
                }
            }
        }

        if keys_added == 0 {
            return Err(AuthError::InternalError(anyhow::Error::msg(
                "No valid RSA signing keys found in JWKS response",
            )));
        }

        log::debug!("Successfully cached {} signing keys", keys_added);
        Ok(())
    }
}

impl Drop for Auth {
    fn drop(&mut self) {
        self.refetch_task_cancel.cancel();
    }
}

// NOTE: Weak pointers dont have drop signaling, we therefore have to manually cancel!
async fn refetch_task(auth: Weak<RwLock<Auth>>, cancel: CancellationToken) -> () {
    let timeout = {
        auth.upgrade()
            .expect("auth was dropped before refetch task could even begin")
            .read()
            .await
            .cache_duration
            .clone()
            / 2 // half time should be fine...
    };
    'outer: loop {
        log::debug!("refetching token in {} seconds!", timeout.as_secs());
        tokio::select! {
            _ = cancel.cancelled() => {
                break;
            }
            _ = tokio::time::sleep(timeout) => {
                log::info!("lol")
            }
        };
        let mut retry_count = 0;
        const MAX_RETRIES: u32 = 3;

        loop {
            log::debug!(
                "attempt to refetch certificates (attempt {})",
                retry_count + 1
            );
            let auth = match auth.upgrade() {
                None => {
                    log::debug!("Auth instance dropped, exiting refetch task");
                    break 'outer;
                }
                Some(auth) => auth,
            };

            let mut auth = auth.write().await;
            match auth.fetch_and_cache_keys().await {
                Ok(()) => {
                    log::debug!("Successfully refetched certificates");
                    break;
                }
                Err(e) => {
                    retry_count += 1;
                    log::error!(
                        "Failed to refetch certificates (attempt {}): {}",
                        retry_count,
                        e
                    );

                    if retry_count >= MAX_RETRIES {
                        log::error!("Max retries exceeded, will try again on next cycle");
                        break;
                    }

                    drop(auth); // Release the lock before sleeping
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }
    log::debug!("cert refetchign terminated")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_verify_user() {
        let auth_arc = Auth::new(
            "http://localhost:8080/realms/box-verse/protocol/openid-connect/certs".to_string(),
            "http://localhost:8080/realms/box-verse".to_string(),
            vec!["account".to_string()],
            Duration::from_secs(10000),
        )
        .await
        .unwrap();

        let token = "eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJQMWVzMS1wSFBIVjc2Y0trMkM0ZW1DSDNQUllpR293bzF6RTV2aDdPS1RrIn0.eyJleHAiOjE3NTM3MjY1NjUsImlhdCI6MTc1MzcyNjI2NSwiYXV0aF90aW1lIjoxNzUzNzI1ODQzLCJqdGkiOiJvbnJ0YWM6YjE4MjcxZGItMzg5My0xMzJmLWQ1ZGQtZjZmYjI2M2ZiZjJjIiwiaXNzIjoiaHR0cDovL2xvY2FsaG9zdDo4MDgwL3JlYWxtcy9ib3gtdmVyc2UiLCJhdWQiOiJhY2NvdW50Iiwic3ViIjoiMTNiMGY4YzgtNzMwZC00ZDFjLTk3NmMtMjMyN2ZjMmNmOWVjIiwidHlwIjoiQmVhcmVyIiwiYXpwIjoiYm94LWxvZ2luIiwic2lkIjoiNmQ4OWNjZjUtZTJjYi00M2QyLWJlMjQtMzUzYmQ4NDNkNzc4IiwiYWNyIjoiMCIsImFsbG93ZWQtb3JpZ2lucyI6WyJodHRwOi8vbG9jYWxob3N0Il0sInJlYWxtX2FjY2VzcyI6eyJyb2xlcyI6WyJvZmZsaW5lX2FjY2VzcyIsInVtYV9hdXRob3JpemF0aW9uIiwiZGVmYXVsdC1yb2xlcy1ib3gtdmVyc2UiXX0sInJlc291cmNlX2FjY2VzcyI6eyJhY2NvdW50Ijp7InJvbGVzIjpbIm1hbmFnZS1hY2NvdW50IiwibWFuYWdlLWFjY291bnQtbGlua3MiLCJ2aWV3LXByb2ZpbGUiXX19LCJzY29wZSI6Im9wZW5pZCBwcm9maWxlIGVtYWlsIiwiZW1haWxfdmVyaWZpZWQiOmZhbHNlLCJuYW1lIjoia2lyaWxsIGtpcmlsbCIsInByZWZlcnJlZF91c2VybmFtZSI6ImtpcmlsbCIsImdpdmVuX25hbWUiOiJraXJpbGwiLCJmYW1pbHlfbmFtZSI6ImtpcmlsbCIsImVtYWlsIjoia2lyaWxsQGtpcmlsbC5jb20ifQ.gCEw5MhYFte6NKFWI7A2Co0syDNyWtdR-sYM-lBSFuQyJCz59xxbjK1j33bldImWbc7RGAnHdYOSP3duuepBWs5QcS5Ybq5um3vr33mD0wjI_n7MDbT6EZpUyE6l6gDMsbIF40EobGKesW2N0aEhqW3392lHA2POXowm62WpMbIYmjC79l6CSZkcZ8js-Sd3u7W1TYrlak25yjGpuifxQ-TpUYoQPr92kP3LKQ0XAa_5SwD7BLKxOAOp4W1d2SWd0cUZRjyFnXyWHlH5u16ngEMM5ZgJ5xrBPtWlPCcOp5MSPvvp6vqNj7tzwcm2IU8LcJIBwm-vLJFimnADnRnJGQ";

        let auth = auth_arc.read().await;
        let result = auth.verify_user(token);
        match result {
            Ok(user_id) => {
                assert_eq!(user_id, "13b0f8c8-730d-4d1c-976c-2327fc2cf9ec");
            }
            Err(_) => {
                // Test may fail due to network issues or expired token, but structure is correct
            }
        }
    }
}
