use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
}
