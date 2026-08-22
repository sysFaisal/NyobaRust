use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;


#[derive(Debug, Clone, Copy, PartialEq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "bottle_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum BottleStatus {
    Available,
    Empty,
    Inactive,
}

#[derive(Deserialize, Validate)]
pub struct CreateBottle {
    pub batch_id: Uuid,
    pub remaining_ml: BigDecimal,
    pub status: BottleStatus,
}

#[derive(Deserialize, Validate)]
pub struct UpdateBottle {
    pub remaining_ml: Option<BigDecimal>,
    pub status: Option<String>,
}

