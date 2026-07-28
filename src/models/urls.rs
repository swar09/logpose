use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::urls)]

pub struct Urls {
    pub short_code: String,
    pub long_url : String, 
    pub created_by : Uuid,
    pub created_at : NaiveDateTime,
    pub updated_at : NaiveDateTime,
}
