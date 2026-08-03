use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Insertable)]
#[diesel(table_name = crate::schema::url_analytics)]
pub struct NewEntry {
    pub device: Option<String>,

    pub browser: Option<String>,

    pub referer: Option<String>,

    pub short_code: String,

    // pub clicked_at: NaiveDateTime,
    pub ip_address: String,

    pub user_agent: Option<String>,

    pub country_code: Option<String>,
}
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::url_analytics)]

pub struct UrlAnalytics {
    pub id: Uuid,

    pub device: Option<String>,

    pub browser: Option<String>,

    pub referer: Option<String>,

    pub short_code: String,

    pub clicked_at: NaiveDateTime,

    pub ip_address: String,

    pub user_agent: Option<String>,

    pub country_code: Option<String>,
}
