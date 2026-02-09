use argon2::{
    Algorithm, Argon2, Params, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_name: String,
    pub exp: usize,
    pub iat: u64,
}

pub struct AuthService {
    password_hasher: Argon2<'static>,
    token_expiry_duration: chrono::Duration,
    secret: Vec<u8>,
}

impl AuthService {
    pub fn new(token_expiry_duration: chrono::Duration, secret: &[u8]) -> Self {
        let params =
            Params::new(19 * 1024, 2, 1, None).expect("Failed to create Argon2 parameters");
        let password_hasher = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        Self {
            password_hasher,
            token_expiry_duration,
            secret: secret.to_vec(),
        }
    }

    pub fn hash_password(&self, password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self
            .password_hasher
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
        Ok(password_hash)
    }

    pub fn verify_password(&self, password: &str, password_hash: &str) -> bool {
        let parsed_hash = match argon2::PasswordHash::new(password_hash) {
            Ok(hash) => hash,
            Err(_) => return false,
        };
        self.password_hasher
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }

    pub fn generate_token(&self, user_id: &str, user_name: &str) -> String {
        let now = chrono::offset::Utc::now();

        let claims = Claims {
            sub: user_id.to_string(),
            user_name: user_name.to_string(),
            exp: (now + self.token_expiry_duration).timestamp() as usize,
            iat: now.timestamp() as u64,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
        .expect("Failed to encode token")
    }

    pub fn generate_refresh_token(&self, user_id: &str, user_name: &str) -> String {
        let now = chrono::offset::Utc::now();
        // Refresh token живет 30 дней
        let refresh_expiry = chrono::Duration::days(30);

        let claims = Claims {
            sub: user_id.to_string(),
            user_name: user_name.to_string(),
            exp: (now + refresh_expiry).timestamp() as usize,
            iat: now.timestamp() as u64,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
        .expect("Failed to encode refresh token")
    }

    pub fn verify_token(&self, token: &str) -> Option<Claims> {
        let decoding_key = jsonwebtoken::DecodingKey::from_secret(self.secret.as_ref());
        let validation = jsonwebtoken::Validation::default();
        match jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation) {
            Ok(token_data) => Some(token_data.claims),
            Err(_) => None,
        }
    }
}
