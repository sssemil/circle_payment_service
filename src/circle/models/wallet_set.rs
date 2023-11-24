use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WalletSetRequest {
    pub idempotency_key: String,
    pub entity_secret_cipher_text: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WalletSetResponse {
    pub id: Uuid,
    pub custody_type: String,
    pub name: String,
    pub update_date: DateTime<Utc>,
    pub create_date: DateTime<Utc>,
}
