use diesel::prelude::*;

use crate::{
    models::url_analytics::{NewEntry, UrlAnalytics},
    schema::{
        url_analytics,
        url_analytics::{dsl::*, short_code},
    },
};

#[allow(dead_code)]
pub fn create(new_entry: NewEntry, conn: &mut PgConnection) -> Result<UrlAnalytics, diesel::result::Error> {
    diesel::insert_into(url_analytics::table)
        .values(&new_entry)
        .returning(UrlAnalytics::as_returning())
        .get_result(conn)
}

pub fn create_batch(entries: &[NewEntry], conn: &mut PgConnection) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(url_analytics::table).values(entries).execute(conn)
}

#[allow(dead_code)]
pub fn delete(uuid: uuid::Uuid, conn: &mut PgConnection) -> Result<usize, diesel::result::Error> {
    diesel::delete(url_analytics.filter(id.eq(uuid))).execute(conn)
}

pub fn get_by_short_code(code: String, conn: &mut PgConnection) -> Result<Vec<UrlAnalytics>, diesel::result::Error> {
    crate::schema::url_analytics::table
        .filter(short_code.eq(code))
        .select(UrlAnalytics::as_select())
        .load(conn)
}
