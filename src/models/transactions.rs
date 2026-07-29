use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;
use diesel_derive_enum::DbEnum;

#[derive(DbEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Pending,
    Success,
    Failed,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transactions)]
pub struct Transactions {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: i32,
    pub currency_code: String,
    pub status: TransactionStatus,
    pub reference_id: Option<Uuid>,
    pub timestamp: NaiveDateTime,
}
