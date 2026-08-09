use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use uuid::Uuid;

#[derive(DbEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Failed,

    Pending,

    Success,
}

#[allow(dead_code)]
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transactions)]
pub struct Transactions {
    pub id: Uuid,

    pub amount: i32,

    pub status: TransactionStatus,

    pub user_id: Uuid,

    pub timestamp: DateTime<Utc>,

    pub reference_id: Option<Uuid>,

    pub currency_code: String,
}
