use serde::Deserialize;

// Login Request Payload
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String
}
