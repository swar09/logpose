use std::sync::Arc;

use axum::RequestPartsExt;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use jsonwebtoken::DecodingKey;

use crate::AppState;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    Admin,

    Client,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    //    iss : String, // issuer (multiple services can assign tokens)
    pub sub: Uuid, // subject (issued to)

    pub iat: usize, // time of assignment

    pub nbf: usize, // time before token is not valid

    pub exp: usize, // time after which token is not valid

    //    aud : String, // audience (services where token is intended to be used)
    pub role: UserRole, // add by me not mentioned in blog
                    // jti: Uuid, // token id for jwt blocking purposes
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
        let claims = crate::utils::auth::verify_jwt(bearer.token().to_string(), decoding_key)
            .map_err(|_| StatusCode::UNAUTHORIZED)?
            .claims;

        Ok(AuthUser {
            user_id: claims.sub,
        })
    }
}

#[derive(Serialize)]
pub struct LoginData {
    pub user: UserRole,

    pub token: String,
    pub token_type: String,

    pub expires_in: usize,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub login_id: String, // email or user name

    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub data: Option<LoginData>,

    pub success: bool,
}
