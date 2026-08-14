// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "transaction_status"))]
    pub struct TransactionStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TransactionStatus;

    transactions (id) {
        id -> Uuid,
        user_id -> Uuid,
        amount -> Int4,
        #[max_length = 5]
        currency_code -> Varchar,
        status -> TransactionStatus,
        reference_id -> Nullable<Uuid>,
        timestamp -> Timestamptz,
    }
}

diesel::table! {
    url_analytics (id) {
        id -> Uuid,
        #[max_length = 4]
        short_code -> Nullable<Varchar>,
        clicked_at -> Timestamptz,
        #[max_length = 45]
        ip_address -> Varchar,
        #[max_length = 1024]
        user_agent -> Nullable<Varchar>,
        #[max_length = 100]
        browser -> Nullable<Varchar>,
        #[max_length = 100]
        device -> Nullable<Varchar>,
        #[max_length = 10]
        country_code -> Nullable<Varchar>,
        #[max_length = 2048]
        referer -> Nullable<Varchar>,
    }
}

diesel::table! {
    urls (database_id) {
        database_id -> Int4,
        #[max_length = 4]
        short_code -> Nullable<Varchar>,
        #[max_length = 2048]
        long_url -> Varchar,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 100]
        first_name -> Varchar,
        #[max_length = 100]
        last_name -> Varchar,
        #[max_length = 30]
        username -> Varchar,
        #[max_length = 100]
        email -> Varchar,
        #[max_length = 128]
        hashed_password -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(transactions -> users (user_id));
diesel::joinable!(urls -> users (created_by));

diesel::allow_tables_to_appear_in_same_query!(transactions, url_analytics, urls, users,);
