use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::urls)]

pub struct Urls {
    pub long_url: String,

    pub short_code: Option<String>,

    pub created_by: Uuid,
    pub created_at: NaiveDateTime,

    pub updated_at: NaiveDateTime,

    pub database_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::urls)]
pub struct NewUrl<'a> {
    pub long_url: &'a str,

    // pub short_code: &'a str,
    pub created_by: Uuid,
}

#[derive(Deserialize)]
pub struct NewUrlRequest {
    pub long_url: String,

    // pub short_code: String,
    pub created_by: Uuid,
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::urls)]
pub struct UpdateUrl<'c> {
    pub long_url: &'c str,
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::urls)]
pub struct UpdateCode<'b> {
    pub short_code: &'b str,
}
#[derive(Insertable)]
#[diesel(table_name = crate::schema::urls)]
pub struct DatabaseId {
    pub database_id: i32,
}

#[derive(Deserialize)]
pub struct UpdateUrlRequest {
    pub long_url: String,
    pub short_code: Option<String>,
    pub created_by: Uuid,
    pub database_id: i32,
}
