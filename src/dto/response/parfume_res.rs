use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct ParfumeResponse {
    pub id: Uuid,
    pub brands_id: Uuid,
    pub name: String,
    pub concrentration: Option<String>,
    pub description: Option<String>,
}
