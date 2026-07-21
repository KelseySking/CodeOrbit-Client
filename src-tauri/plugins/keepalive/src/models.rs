use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeepAlivePayload {
    pub target_name: String,
    pub pending_count: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingAlertPayload {
    pub title: String,
    pub body: String,
    pub count: i32,
}
