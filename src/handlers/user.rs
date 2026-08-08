use axum::Json;
use axum::extract::State;
use sqlx::PgPool;

use crate::dto::response::response_user::UserProfile;
use crate::error::error::AppError;
use crate::service::service_user;

pub async fn get_all_user(State(pool): State<PgPool>) -> Result<Json<Vec<UserProfile>>, AppError> {
    let users = service_user::svc_get_all_user(&pool).await?;

    Ok(Json(users))
}

pub async fn get_user_by_id() -> &'static str {
    "get all user"
}

pub async fn create_user() -> &'static str {
    "create user"
}

pub async fn update_all_data_user() -> &'static str {
    "create user"
}

pub async fn update_data_user() -> &'static str {
    "user"
}
