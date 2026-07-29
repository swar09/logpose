use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::url_analytics)]

pub struct UrlAnalytics {
    pub id: Uuid,
    pub short_code: String,
    pub clicked_at: NaiveDateTime,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub browser: Option<String>,
    pub device: Option<String>,
    pub country_code: Option<String>,
    pub referer: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::url_analytics)]
pub struct NewEntry {
    pub short_code: String,
    pub clicked_at: NaiveDateTime,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub browser: Option<String>,
    pub device: Option<String>,
    pub country_code: Option<String>,
    pub referer: Option<String>,
} 



