use serde::{Deserialize, Serialize};

// Session Data to store in Redis
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: String,
    pub role: String, // admin, viewer, etc.
}