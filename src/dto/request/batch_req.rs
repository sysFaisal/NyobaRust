use bigdecimal::BigDecimal;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateBatch {
    pub quantity_ml: BigDecimal,
    pub purchase_price: BigDecimal,
}

#[derive(Deserialize, Validate)]
pub struct UpdateBatch {
    pub quantity_ml: Option<BigDecimal>,
    pub purchase_price: Option<BigDecimal>,
}
