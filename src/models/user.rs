use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Selectable, Serialize, Clone, Debug)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Users {
    pub id: Uuid,

    pub first_name: String,

    pub last_name: String,

    pub username: String,

    pub email: String,

    pub hashed_password: String,

    pub avatar_url: Option<String>,

    pub google_id: Option<String>,

    pub auth_provider: String,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser<'a> {
    pub email: &'a str,

    pub username: &'a str,

    pub last_name: &'a str,

    pub first_name: &'a str,

    pub hashed_password: &'a str,

    pub avatar_url: Option<&'a str>,

    pub google_id: Option<&'a str>,

    pub auth_provider: &'a str,
}

#[derive(Deserialize)]
pub struct NewUserRequest {
    pub email: String,

    pub username: String,

    pub password: String,

    pub last_name: String,

    pub first_name: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,

    pub email: String,

    pub username: String,

    pub last_name: String,

    pub first_name: String,

    pub avatar_url: Option<String>,

    pub auth_provider: String,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
}

impl From<Users> for UserResponse {
    fn from(user: Users) -> Self {
        UserResponse {
            id: user.id,
            email: user.email,
            username: user.username,
            last_name: user.last_name,
            first_name: user.first_name,
            avatar_url: user.avatar_url,
            auth_provider: user.auth_provider,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::users)]
pub struct UpdateUser<'b> {
    pub username: &'b str,

    pub last_name: &'b str,

    pub first_name: &'b str,
}

#[derive(Deserialize)]
pub struct UpdateRequest {
    pub username: String,

    pub last_name: String,

    pub first_name: String,
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::users)]
pub struct UpdatePassword<'c> {
    pub hashed_password: &'c str,
}

#[derive(Deserialize)]
pub struct UpdatePasswordRequest {
    pub old_password: String,

    pub new_password: String,
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::users)]
pub struct UpdateOAuthProfile<'d> {
    pub first_name: &'d str,

    pub last_name: &'d str,

    pub avatar_url: Option<&'d str>,

    pub google_id: Option<&'d str>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,

    pub username: String,

    pub password: String,

    pub last_name: String,

    pub first_name: String,
}
