use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dto::request::botol_req::BottleStatus;

#[derive(Serialize, Deserialize)]
pub struct BotolResponse {
    pub id: Uuid,
    pub batch_parfume_id: Uuid,
    pub remaining_ml: BigDecimal,
    pub status: BottleStatus,
}
