use serde::{Deserialize, Serialize};

// Define the structure for the response
#[derive(Serialize, Deserialize)]
pub struct UserRoleResponse {
    pub role: String,
    pub message: String,
}
