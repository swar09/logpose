use argon2::password_hash::PasswordVerifier;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher};
use rand_core::OsRng;
use uuid::Uuid;

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

use crate::models::auth::{Claims, UserRole};

const DURATION: u64 = 3600;

pub fn genrate_jwt(
    role: UserRole,
    user_id: Uuid,
    key: EncodingKey,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now().timestamp() as u64;
    let jti = Uuid::new_v4();
    let claims = Claims {
        sub: user_id,
        role,
        iat: now,
        nbf: now,
        exp: now + DURATION,
        jti,
    };
    let header = Header::new(jsonwebtoken::Algorithm::HS256);

    encode(&header, &claims, &key)
}

pub fn verify_jwt(
    token: String,
    key: DecodingKey,
) -> Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> {
    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    decode(token, &key, &validation)
}

pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(stored_hash) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn hash_password(password: String) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing_and_verification() {
        let password = "MySecretPassword123!".to_string();
        let hash = hash_password(password.clone()).expect("Failed to hash password");

        assert!(verify_password(&password, &hash));
        assert!(!verify_password("WrongPassword123!", &hash));
    }

    #[test]
    fn test_jwt_generation_and_verification() {
        let _ = jsonwebtoken::crypto::CryptoProvider::install_default(
            &jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER,
        );

        let secret = "test_jwt_secret_key_1234567890!@#$";
        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());

        let user_id = Uuid::new_v4();
        let token = genrate_jwt(UserRole::Client, user_id, encoding_key)
            .expect("Failed to generate JWT");

        let token_data = verify_jwt(token, decoding_key).expect("Failed to verify JWT");
        assert_eq!(token_data.claims.sub, user_id);
        assert_eq!(token_data.claims.role, UserRole::Client);
    }

    #[test]
    fn test_jwt_verification_fails_with_invalid_secret() {
        let _ = jsonwebtoken::crypto::CryptoProvider::install_default(
            &jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER,
        );

        let secret = "correct_secret_key_1234567890";
        let invalid_secret = "wrong_secret_key_1234567890!!";

        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(invalid_secret.as_bytes());

        let user_id = Uuid::new_v4();
        let token = genrate_jwt(UserRole::Client, user_id, encoding_key).unwrap();

        assert!(verify_jwt(token, decoding_key).is_err());
    }
}
