use diesel::prelude::*;
use crate::{
    models::url_analytics::{NewEntry, UrlAnalytics},
    schema::url_analytics,
};

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

    diesel::delete(url_analytics.filter(id.eq(uuid)))
        .execute(conn)
}