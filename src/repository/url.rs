use chrono::{DateTime, Utc};
use diesel::ExpressionMethods;
use diesel::{BoolExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;

use crate::models::url::{NewUrl, UpdateCode, UpdateUrl, Urls};
use crate::schema::urls::database_id;
use crate::schema::urls::short_code;
use crate::schema::urls::{self, created_by};

pub fn create(new_url: NewUrl, conn: &mut PgConnection) -> Result<Urls, diesel::result::Error> {
    diesel::insert_into(urls::table)
        .values(new_url)
        .returning(Urls::as_returning())
        .get_result(conn)
}

pub fn delete_by_short_code(
    code: String,
    conn: &mut PgConnection,
) -> Result<usize, diesel::result::Error> {
    diesel::delete(urls::table.filter(short_code.eq(code))).execute(conn)
}

pub fn modify_url_by_id(
    id: i32,
    new_long_url: UpdateUrl,
    conn: &mut PgConnection,
) -> Result<usize, diesel::result::Error> {
    diesel::update(urls::table.find(id))
        .set(new_long_url)
        .execute(conn)
}

pub fn modify_code_by_id(
    id: i32,
    new_short_code: UpdateCode,
    conn: &mut PgConnection,
) -> Result<usize, diesel::result::Error> {
    diesel::update(urls::table.find(id))
        .set(new_short_code)
        .execute(conn)
}

#[allow(dead_code)]
pub fn get_by_id(id: i32, conn: &mut PgConnection) -> Result<Urls, diesel::result::Error> {
    urls::table
        .filter(database_id.eq(id))
        .select(Urls::as_select())
        .first(conn)
}

pub fn get_by_short_code(
    code: String,
    conn: &mut PgConnection,
) -> Result<Urls, diesel::result::Error> {
    urls::table
        .filter(short_code.eq(code))
        .select(crate::models::url::Urls::as_select())
        .first(conn)
}

pub fn get_long_url_by_id(
    id: i32,
    conn: &mut PgConnection,
) -> Result<String, diesel::result::Error> {
    urls::table
        .filter(database_id.eq(id))
        .filter(
            urls::expires_at
                .is_null()
                .or(urls::expires_at.gt(diesel::dsl::now)),
        )
        .select(urls::long_url)
        .first(conn)
}

pub fn get_urls_by_user_id(
    id: Uuid,
    conn: &mut PgConnection,
) -> Result<Vec<Urls>, diesel::result::Error> {
    urls::table
        .filter(created_by.eq(id))
        .select(crate::models::url::Urls::as_select())
        .load(conn)
}

pub fn count_urls_by_user_id(
    user_id_val: Uuid,
    conn: &mut PgConnection,
) -> Result<i64, diesel::result::Error> {
    urls::table
        .filter(created_by.eq(Some(user_id_val)))
        .count()
        .get_result(conn)
}

pub fn check_short_code_exists(
    code: &str,
    conn: &mut PgConnection,
) -> Result<bool, diesel::result::Error> {
    use diesel::dsl::exists;
    use diesel::select;
    select(exists(urls::table.filter(short_code.eq(code)))).get_result(conn)
}

pub fn get_user_id_by_short_code(
    code: String,
    conn: &mut PgConnection,
) -> Result<Option<Uuid>, diesel::result::Error> {
    urls::table
        .filter(short_code.eq(code))
        .select(created_by)
        .first(conn)
}

pub fn get_long_url_by_short_code(
    code: String,
    conn: &mut PgConnection,
) -> Result<String, diesel::result::Error> {
    urls::table
        .filter(short_code.eq(code))
        .filter(
            urls::expires_at
                .is_null()
                .or(urls::expires_at.gt(diesel::dsl::now)),
        )
        .select(urls::long_url)
        .first(conn)
}

pub fn get_active_urls_by_guest_id(
    guest_id_val: Uuid,
    conn: &mut PgConnection,
) -> Result<Vec<Urls>, diesel::result::Error> {
    urls::table
        .filter(urls::guest_id.eq(guest_id_val))
        .filter(
            urls::expires_at
                .is_null()
                .or(urls::expires_at.gt(diesel::dsl::now)),
        )
        .order(urls::created_at.desc())
        .select(Urls::as_select())
        .load(conn)
}

pub fn claim_guest_urls(
    guest_id_val: Uuid,
    user_id_val: Uuid,
    conn: &mut PgConnection,
) -> Result<usize, diesel::result::Error> {
    diesel::update(
        urls::table.filter(urls::guest_id.eq(guest_id_val)).filter(
            urls::expires_at
                .is_null()
                .or(urls::expires_at.gt(diesel::dsl::now)),
        ),
    )
    .set((
        urls::created_by.eq(Some(user_id_val)),
        urls::guest_id.eq(None::<Uuid>),
        urls::expires_at.eq(None::<DateTime<Utc>>),
    ))
    .execute(conn)
}

pub fn cleanup_expired_guest_urls(conn: &mut PgConnection) -> Result<usize, diesel::result::Error> {
    diesel::delete(
        urls::table
            .filter(urls::guest_id.is_not_null())
            .filter(urls::expires_at.lt(diesel::dsl::now)),
    )
    .execute(conn)
}
