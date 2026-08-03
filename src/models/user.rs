use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]

pub struct Users {
    pub id: Uuid,

    pub email: String,

    pub username: String,

    pub last_name: String,

    pub first_name: String,

    pub created_at: NaiveDateTime,

    pub updated_at: NaiveDateTime,

    pub hashed_password: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser<'a> {
    pub email: &'a str,

    pub username: &'a str,

    pub last_name: &'a str,

    pub first_name: &'a str,

    pub hashed_password: &'a str,
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
    pub email: String,

    pub username: String,

    pub last_name: String,

    pub first_name: String,
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
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,

    pub username: String,

    pub password: String,

    pub last_name: String,

    pub first_name: String,
}
