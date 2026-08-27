use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateOrder {
    pub bottle_id: Uuid,
    pub decant_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct OrderPageQuery {
    pub page: Option<i64>,
}
