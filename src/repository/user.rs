use crate::models::users::{NewUser, UpdateUser};
use crate::models::users::{UpdatePassword, Users};
use crate::schema::users::{self, id};
use diesel::PgConnection;
use diesel::RunQueryDsl;
use diesel::SelectableHelper;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use uuid::Uuid;

pub fn create(conn: &mut PgConnection, user: &NewUser) -> Result<Users, diesel::result::Error> {
    diesel::insert_into(users::table)
        .values(user)
        .returning(Users::as_returning())
        .get_result(conn)
}

pub fn update_by_id(
    conn: &mut PgConnection,
    user_id: Uuid,
    updated_user: &UpdateUser,
) -> Result<Users, diesel::result::Error> {
    diesel::update(users::table.find(user_id))
        .set(updated_user)
        .returning(Users::as_returning())
        .get_result(conn)
}

pub fn delete_by_id(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<usize, diesel::result::Error> {
    diesel::delete(users::table.find(user_id)).execute(conn)
}

pub fn update_password_by_id(
    conn: &mut PgConnection,
    user_id: Uuid,
    updated_password: UpdatePassword,
) -> Result<usize, diesel::result::Error> {
    diesel::update(users::table.find(user_id))
        .set(updated_password)
        .execute(conn)
}

pub fn get_by_id(user_id : Uuid ,conn: &mut PgConnection) -> Result<Users, diesel::result::Error> {
    users::table.filter(id.eq(user_id)).select(Users::as_select()).first(conn)
}