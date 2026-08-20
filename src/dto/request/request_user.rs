use serde::Deserialize;
use serde_email::Email;
use uuid::Uuid;
use validator::Validate;
use validator::ValidationError;

#[derive(Deserialize, Validate)]
pub struct CreateUser {
    #[validate(length(min = 3))]
    pub username: String,
    pub email: Option<Email>,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct LoginUser {
    #[validate(length(min = 3))]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct CreateBrands {
    #[validate(length(min = 3))]
    pub name_brands: String,
}

#[derive(Deserialize, Validate)]
pub struct Parfume {
    pub brands_id: Uuid,
    #[validate(length(min = 3))]
    pub name: String,
    pub concrentration: Option<String>,
    pub description: Option<String>,
}
