//! Card payment routes
//!
//! These routes are only registered when CARD_ENABLED=true

use axum::{routing::post, Router};
use serde_json::json;
use std::sync::Arc;

use crate::state::AppState;

/// Create card routes router
pub fn create_card_router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/card/webhook", post(card_webhook))
        .with_state(state)
}

/// Card webhook handler for partner events
async fn card_webhook(
    axum::extract::State(_state): axum::extract::State<Arc<AppState>>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> Result<axum::Json<serde_json::Value>, crate::error::ApiError> {
    use crate::card::webhooks::{deserialize_partner_event, PartnerEvent};

    let payload_str = serde_json::to_string(&payload)
        .map_err(|e| crate::error::ApiError::BadRequest(format!("Invalid JSON payload: {}", e)))?;

    let event = deserialize_partner_event(&payload_str)?;

    // Process the event based on type
    match event {
        PartnerEvent::Authorization(ref auth) => {
            tracing::info!(
                event_id = %auth.event_id,
                partner_txn_id = %auth.partner_transaction_id,
                amount = auth.amount,
                currency = %auth.currency,
                "Card authorization received"
            );
        }
        PartnerEvent::Clearing(ref clearing) => {
            tracing::info!(
                event_id = %clearing.event_id,
                partner_txn_id = %clearing.partner_transaction_id,
                amount = clearing.amount,
                currency = %clearing.currency,
                "Card clearing received"
            );
        }
        PartnerEvent::Reversal(ref reversal) => {
            tracing::info!(
                event_id = %reversal.event_id,
                partner_txn_id = %reversal.partner_transaction_id,
                amount = reversal.amount,
                currency = %reversal.currency,
                "Card reversal received"
            );
        }
    }

    let event_id = match event {
        PartnerEvent::Authorization(a) => a.event_id,
        PartnerEvent::Clearing(c) => c.event_id,
        PartnerEvent::Reversal(r) => r.event_id,
    };

    Ok(axum::Json(json!({
        "status": "accepted",
        "event_id": event_id
    })))
}
