use uuid::Uuid;
use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct CreateBrands {
    #[validate(length(min = 3))]
    pub name_brands: String,
}

#[derive(serde::Deserialize, Validate)]
pub struct UpdateBrands {
    pub brands_id: Uuid,
    #[validate(length(min = 3))]
    pub name_brands: String,
}
