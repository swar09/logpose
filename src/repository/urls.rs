use crate::{
    models::urls::{NewUrl, UpdateCode, UpdateUrl, Urls},
    schema::urls,
};
use diesel::{PgConnection, RunQueryDsl, SelectableHelper, query_dsl::methods::FindDsl};

pub fn create(new_url: NewUrl, conn: &mut PgConnection) -> Result<Urls, diesel::result::Error> {
    diesel::insert_into(urls::table)
        .values(new_url)
        .returning(Urls::as_returning())
        .get_result(conn)
}
pub fn delete(short_code: String, conn: &mut PgConnection) -> Result<usize, diesel::result::Error> {
    diesel::delete(urls::table.find(short_code)).execute(conn)
}
pub fn modify_code(
    old_short_code: String,
    new_short_code: UpdateCode,
    conn: &mut PgConnection,
) -> Result<usize, diesel::result::Error> {
    diesel::update(urls::table.find(old_short_code))
        .set(new_short_code)
        .execute(conn)
}

pub fn modify_url(
    short_code: String,
    new_long_url: UpdateUrl,
    conn: &mut PgConnection,
) -> Result<usize, diesel::result::Error> {
    diesel::update(urls::table.find(short_code))
        .set(new_long_url)
        .execute(conn)
}
