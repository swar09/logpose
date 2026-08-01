use crate::AppState;
use argon2::password_hash::PasswordVerifier;
use argon2::{Argon2, PasswordHash, PasswordHasher, password_hash::SaltString};
use axum::RequestPartsExt;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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
pub struct LoginResponse {
    pub success: bool,
    pub data: Option<LoginData>,
}

#[derive(Serialize)]
pub struct LoginData {
    pub user: UserRole,
    pub token: String,
    pub token_type: String,
    pub expires_in: usize,
}

const DURATION: usize = 3600;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
pub fn genrate_jwt(
    role: UserRole,
    user_id: Uuid,
    key: EncodingKey,
) -> Result<String, jsonwebtoken::errors::Error> {
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
    key: DecodingKey,
) -> Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> {
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
                    Ok(string_pasrsed_hash)
                }
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
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

pub struct AuthUser {
    pub user_id: Uuid,
}
impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;
        let decoding_key = DecodingKey::from_secret(state.jwt_secret.as_bytes());
        let claims = verify_jwt(bearer.token().to_string(), decoding_key)
            .map_err(|_| StatusCode::UNAUTHORIZED)?
            .claims;

        Ok(AuthUser {
            user_id: claims.sub,
        })
    }
}
