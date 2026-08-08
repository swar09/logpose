use argon2::password_hash::PasswordVerifier;
use argon2::{Argon2, PasswordHash, PasswordHasher, password_hash::SaltString};
use rand_core::OsRng;
use uuid::Uuid;

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

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
    let result: Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> =
        decode(token, &key, &validation);
    result
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
    let password_as_bytes = password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let result = argon2.hash_password(password_as_bytes, &salt);
    match result {
        Ok(password_hash) => {
            let string_pasword_hash = password_hash.to_string();
            let result = PasswordHash::new(&string_pasword_hash);
            match result {
                Ok(parsed_hash) => {
                    let string_pasrsed_hash = parsed_hash.to_string();
                    Ok(string_pasrsed_hash)
                }
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}
