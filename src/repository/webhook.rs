use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::billing::{NewWebhookEvent, WebhookEvent, WebhookProcessingStatus};
use crate::schema::webhook_events;

pub fn record_webhook_event(
    new_event: NewWebhookEvent,
    conn: &mut PgConnection,
) -> Result<Option<Uuid>, diesel::result::Error> {
    diesel::insert_into(webhook_events::table)
        .values(&new_event)
        .on_conflict(webhook_events::event_id)
        .do_nothing()
        .returning(webhook_events::id)
        .get_result::<Uuid>(conn)
        .optional()
}

pub fn get_webhook_event_by_event_id(
    event_id_str: &str,
    conn: &mut PgConnection,
) -> Result<WebhookEvent, diesel::result::Error> {
    webhook_events::table
        .filter(webhook_events::event_id.eq(event_id_str))
        .select(WebhookEvent::as_select())
        .first(conn)
}

pub fn mark_webhook_processed(
    event_id_str: &str,
    status_val: WebhookProcessingStatus,
    error_log_val: Option<&str>,
    conn: &mut PgConnection,
) -> Result<usize, diesel::result::Error> {
    let now = Utc::now();
    diesel::update(webhook_events::table.filter(webhook_events::event_id.eq(event_id_str)))
        .set((
            webhook_events::status.eq(status_val),
            webhook_events::error_log.eq(error_log_val),
            webhook_events::processed_at.eq(Some(now)),
        ))
        .execute(conn)
}
