use serde::{Deserialize, Serialize};

// Login Request Payload
#[derive(Debug, Deserialize, Serialize)]
pub struct LogoutResponse {
    pub username: String,
    pub message: String,
}
