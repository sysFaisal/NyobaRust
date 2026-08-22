use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct ParfumeResponse {
    pub id: Uuid,
    pub brands_id: Uuid,
    pub brands_name: String,
    pub name: String,
    pub concentration: Option<String>,
    pub description: Option<String>,
}

