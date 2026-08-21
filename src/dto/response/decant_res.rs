use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct DecantResponse {
    pub id: Uuid,
    pub parfume_id: Uuid,
    pub size_ml: i32,
    pub sell_price: BigDecimal,
    pub is_active: bool,
}
