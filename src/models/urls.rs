use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::urls)]

pub struct Urls {
    pub short_code: String,
    pub long_url: String,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::urls)]
pub struct NewUrl<'a> {
    pub short_code: &'a str,
    pub long_url: &'a str,
    pub created_by: Uuid,
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::urls)]
pub struct UpdateCode<'b> {
    pub short_code: &'b str,
}
#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::urls)]
pub struct UpdateUrl<'c> {
    pub long_url: &'c str,
}
