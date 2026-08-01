use argon2::password_hash::PasswordVerifier;
use argon2::{Argon2, PasswordHash, PasswordHasher, password_hash::SaltString};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    //    iss : String, // issuer (multiple services can assign tokens)
    sub: Uuid, // subject (issued to)
    //    aud : String, // audience (services where token is intended to be used)
    role: UserRole, // add by me not mentioned in blog
    iat: usize,     // time of assignment
    nbf: usize,     // time before token is not valid
    exp: usize,     // time after which token is not valid
                    // jti: Uuid, // token id for jwt blocking purposes
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub login_id: String, // email or user name
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Client,
}

#[derive(Serialize)]
struct LoginResponse {
    success: bool,
    data: Option<LoginData>,
}

#[derive(Serialize)]
struct LoginData {
    user: UserRole,
    token: String,
    token_type: String,
    expires_in: usize,
}

const DURATION: usize = 3600;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
pub fn genrate_jwt(role: UserRole, user_id: Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = b"key";
    let key = EncodingKey::from_secret(secret);

    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id,
        role,
        iat: now,
        nbf: now,
        exp: now + DURATION,
    };
    let header = Header::new(jsonwebtoken::Algorithm::HS256);

    encode(&header, &claims, &key)
}
pub fn verify_jwt(
    token: String,
) -> Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> {
    let secret = b"key";
    let key = DecodingKey::from_secret(secret);
    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

    let result: Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> =
        decode(token, &key, &validation);
    result
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
                    return Ok(string_pasrsed_hash);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
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
