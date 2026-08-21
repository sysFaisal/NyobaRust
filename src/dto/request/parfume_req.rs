use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateParfume {
    pub brands_id: Uuid,
    #[validate(length(min = 3))]
    pub name: String,
    pub concrentration: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateParfume {
    pub brands_id: Option<Uuid>,
    #[validate(length(min = 3))]
    pub name: Option<String>,
    pub concrentration: Option<Option<String>>,
    pub description: Option<Option<String>>,
}
