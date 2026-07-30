use crate::{
    models::url_analytics::{NewEntry, UrlAnalytics},
    schema::url_analytics,
};
use diesel::prelude::*;

pub fn create(
    new_entry: NewEntry,
    conn: &mut PgConnection,
) -> Result<UrlAnalytics, diesel::result::Error> {
    diesel::insert_into(url_analytics::table)
        .values(&new_entry)
        .returning(UrlAnalytics::as_returning())
        .get_result(conn)
}

pub fn delete(uuid: uuid::Uuid, conn: &mut PgConnection) -> Result<usize, diesel::result::Error> {
    use crate::schema::url_analytics::dsl::*;

    diesel::delete(url_analytics.filter(id.eq(uuid))).execute(conn)
}

pub fn get_by_short_code(
    code: String,
    conn: &mut PgConnection,
) -> Result<Vec<UrlAnalytics>, diesel::result::Error> {
    use crate::models::url_analytics::UrlAnalytics;
    use crate::schema::url_analytics::short_code;
    crate::schema::url_analytics::table
        .filter(short_code.eq(code))
        .select(UrlAnalytics::as_select())
        .load(conn)
}
