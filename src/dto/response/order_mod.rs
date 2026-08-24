use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Success,
    Failed,
    Pending,
}

pub struct OrderValue {
    pub bottle_id: Uuid,
    pub decant_id: Uuid,
    pub total_price: BigDecimal,
    pub quantity: i32,
    pub price: BigDecimal,
    pub status: OrderStatus,
}

#[derive(Serialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub bottle_id: Uuid,
    pub decant_id: Uuid,
    pub total_price: BigDecimal,
    pub quantity: i32,
    pub price: BigDecimal,
    pub status: OrderStatus,
    pub size_ml: i32,
    pub parfume_name: String,
    pub brands_name: String,
    pub created_at: DateTime<Utc>,
}
