use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Brand {
    pub id: Uuid,
    pub name: String,
    pub total_parfume: Option<i32>,
}
