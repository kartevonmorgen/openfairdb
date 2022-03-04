use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use jwt_service::JwtService;
use parking_lot::{Mutex, MutexGuard};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// The email in our case
    sub: String,
    /// Expiry time as Unix timestamp
    exp: usize,
}

pub struct JwtState {
    jwt_service: JwtService,
    time_valid: Duration,
    blacklist: Mutex<HashSet<String>>,
}

impl JwtState {
    pub fn new() -> Self {
        Self {
            jwt_service: JwtService::new(),
            time_valid: Duration::days(1),
            blacklist: Mutex::new(HashSet::new()),
        }
    }

    pub fn generate_token(&self, email: &str) -> Result<String> {
        let exp = usize::try_from((Utc::now() + self.time_valid).timestamp())?;
        let claims = Claims {
            sub: email.to_string(),
            exp,
        };
        let token = self.jwt_service.encode(&claims)?;
        Ok(token)
    }

    pub fn validate_token_and_get_email(&self, token: &str) -> Result<String> {
        if self.is_on_blacklist(token) {
            return Err(anyhow!("Token is no longer valid"));
        }
        let claims = self.jwt_service.decode(token)?;
        let sub = claims.sub;
        Ok(sub)
    }

    pub fn blacklist_token(&self, token: String) {
        self.remove_invalid_tokens(); // do housekeeping
        self.lock().insert(token);
    }

    fn is_on_blacklist(&self, token: &str) -> bool {
        self.lock().get(token).is_some()
    }

    // TODO: maybe this can be done more efficiently
    fn remove_invalid_tokens(&self) {
        let invalid_tokens = self
            .lock()
            .iter()
            .filter(|token| self.jwt_service.decode(token).is_err())
            .cloned()
            .collect::<Vec<_>>();
        for token in invalid_tokens {
            self.lock().remove(&token);
        }
    }

    fn lock(&self) -> MutexGuard<HashSet<String>> {
        self.blacklist.lock()
    }
}

#[cfg(feature = "jwt")]
mod jwt_service {
    use super::{Claims, Result};
    use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

    /// generate a Rocket-compatible secret (Rocket expects a
    /// 256-bit base64 encoded string)
    fn generate_rocket_secret() -> String {
        base64::encode(&rand::random::<[u8; 32]>())
    }

    pub struct Key {
        encoding_key: EncodingKey,
        decoding_key: DecodingKey,
    }

    impl Key {
        pub fn new(secret: String) -> Self {
            let encoding_key = EncodingKey::from_secret(secret.as_ref());
            let decoding_key = DecodingKey::from_secret(secret.as_ref());
            Self {
                encoding_key,
                decoding_key,
            }
        }

        pub fn random() -> Self {
            let secret = generate_rocket_secret();
            Self::new(secret)
        }
    }

    pub struct JwtService {
        key: Key,
    }

    impl JwtService {
        pub fn new() -> Self {
            Self { key: Key::random() }
        }
        pub fn encode(&self, claims: &Claims) -> Result<String> {
            let token = encode(&Header::default(), claims, &self.key.encoding_key)?;
            Ok(token)
        }
        pub fn decode(&self, token: &str) -> Result<Claims> {
            let token_data =
                decode::<Claims>(token, &self.key.decoding_key, &Validation::default())?;
            Ok(token_data.claims)
        }
    }
}

#[cfg(not(feature = "jwt"))]
mod jwt_service {
    use super::{Claims, Result};
    pub struct JwtService;
    impl JwtService {
        pub fn new() -> Self {
            Self {}
        }
        pub fn encode(&self, _claims: &Claims) -> Result<String> {
            unimplemented!()
        }
        pub fn decode(&self, _token: &str) -> Result<Claims> {
            unimplemented!()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "jwt")]
mod tests {
    use super::*;

    #[test]
    fn blacklisting_works() {
        let jwt_state = JwtState::new();
        let token = jwt_state.generate_token("foo@bar.org").unwrap();
        jwt_state.blacklist_token(token.clone());
        assert!(jwt_state.is_on_blacklist(&token));
    }

    #[test]
    fn validation_works() {
        let jwt_state = JwtState::new();
        let token = jwt_state.generate_token("foo@bar.org").unwrap();
        let email = jwt_state.validate_token_and_get_email(&token).unwrap();
        assert_eq!(email, "foo@bar.org");
        jwt_state.blacklist_token(token.clone());
        assert!(jwt_state.validate_token_and_get_email(&token).is_err())
    }

    #[test]
    fn invalid_tokens_are_removed() {
        let jwt_state = JwtState::new();
        let token = jwt_state.generate_token("foo@bar.org").unwrap();
        let invalid_token = "dubidubidu".to_string();
        jwt_state.blacklist_token(token.clone());
        jwt_state.blacklist_token(invalid_token.clone());
        assert!(jwt_state.is_on_blacklist(&token));
        assert!(jwt_state.is_on_blacklist(&invalid_token));
        jwt_state.remove_invalid_tokens();
        assert!(jwt_state.is_on_blacklist(&token));
        assert!(!jwt_state.is_on_blacklist(&invalid_token));
    }
}
