use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    //    iss : String, // issuer (multiple services can assign tokens)
       sub : Uuid, // subject (issued to)
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
pub struct LoginResponse {}

const DURATION: usize = 3600;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
pub fn genrate_jwt(role: UserRole, user_id : Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = b"key";
    let key = EncodingKey::from_secret(secret);

    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub : user_id,
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


pub fn hash_password() {}
pub fn verify_password() {}
