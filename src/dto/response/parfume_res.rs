use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
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

#[derive(Debug, Serialize)]
pub struct ParfumeRankingPoint {
    pub parfume_id: Uuid,
    pub parfume_name: String,
    pub brands_name: String,
    pub total_revenue: BigDecimal,
    pub total_cost: BigDecimal,
    pub estimated_profit: BigDecimal,
    pub total_orders: i64,
    pub rank: i64,
}

