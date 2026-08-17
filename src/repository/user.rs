use diesel::ExpressionMethods;
use diesel::PgConnection;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::SelectableHelper;
use uuid::Uuid;

use crate::models::user::{NewUser, UpdateOAuthProfile, UpdatePassword, UpdateUser, Users};
use crate::schema::users::{self, email, google_id, hashed_password, id};

pub fn create(conn: &mut PgConnection, user: &NewUser) -> Result<Users, diesel::result::Error> {
    diesel::insert_into(users::table)
        .values(user)
        .returning(Users::as_returning())
        .get_result(conn)
}

pub fn delete_by_id(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<usize, diesel::result::Error> {
    diesel::delete(users::table.find(user_id)).execute(conn)
}

pub fn update_by_id(
    conn: &mut PgConnection,
    user_id: Uuid,
    updated_user_data: &UpdateUser,
) -> Result<Users, diesel::result::Error> {
    diesel::update(users::table.find(user_id))
        .set(updated_user_data)
        .returning(Users::as_returning())
        .get_result(conn)
}

pub fn update_password_by_id(
    conn: &mut PgConnection,
    user_id: Uuid,
    updated_hashed_password: UpdatePassword,
) -> Result<usize, diesel::result::Error> {
    diesel::update(users::table.find(user_id))
        .set(updated_hashed_password)
        .execute(conn)
}

pub fn get_by_id(user_id: Uuid, conn: &mut PgConnection) -> Result<Users, diesel::result::Error> {
    users::table
        .filter(id.eq(user_id))
        .select(Users::as_select())
        .first(conn)
}

pub fn get_by_email(
    user_email: &str,
    conn: &mut PgConnection,
) -> Result<Users, diesel::result::Error> {
    users::table
        .filter(email.eq(user_email))
        .select(Users::as_select())
        .first(conn)
}

pub fn get_by_google_id(
    google_id_val: &str,
    conn: &mut PgConnection,
) -> Result<Users, diesel::result::Error> {
    users::table
        .filter(google_id.eq(Some(google_id_val)))
        .select(Users::as_select())
        .first(conn)
}

pub fn update_oauth_profile(
    user_id: Uuid,
    profile: &UpdateOAuthProfile,
    conn: &mut PgConnection,
) -> Result<Users, diesel::result::Error> {
    diesel::update(users::table.find(user_id))
        .set(profile)
        .returning(Users::as_returning())
        .get_result(conn)
}

pub fn get_hashed_password_by_id(
    user_id: Uuid,
    conn: &mut PgConnection,
) -> Result<String, diesel::result::Error> {
    users::table
        .filter(id.eq(user_id))
        .select(hashed_password)
        .first(conn)
}
