use serde::Deserialize;
use serde_email::Email;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateUser {
    #[validate(length(min = 3))]
    pub username: String,
    pub email: Option<Email>,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct UpdateUser {
    #[validate(length(min = 3))]
    pub username: Option<String>,
    pub email: Option<Option<Email>>,
    #[validate(length(min = 8))]
    pub password: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct LoginUser {
    #[validate(length(min = 3))]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
}
