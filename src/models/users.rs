use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]

pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}
