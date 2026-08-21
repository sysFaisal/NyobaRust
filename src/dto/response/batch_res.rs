use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct BatchResponse {
    pub id: Uuid,
    pub parfume_id: Uuid,
    pub quantity_ml: BigDecimal,
    pub purchase_price: BigDecimal,
}
