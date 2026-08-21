use bigdecimal::BigDecimal;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateDecant {
    pub parfume_id: Uuid,
    pub size_ml: i32,
    pub sell_price: BigDecimal,
    pub is_active: bool,
}

#[derive(Deserialize, Validate)]
pub struct UpdateDecant {
    pub size_ml: Option<i32>,
    pub sell_price: Option<BigDecimal>,
    pub is_active: Option<bool>,
}
