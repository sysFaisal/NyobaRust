use std::result;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::request::request_user::CreateUser;
use crate::dto::response::response_user::UserProfile;
use crate::error::error::AppError;
use crate::service::service_user;

pub async fn get_all_user(
    State(pool): State<PgPool>,
) -> Result<(StatusCode, Json<Vec<UserProfile>>), AppError> {
    let users = service_user::svc_get_all_user(&pool).await?;

    Ok((StatusCode::OK, Json(users)))
}

pub async fn get_user_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<UserProfile>), AppError> {
    let user = service_user::svc_get_user_by_id(&pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok((StatusCode::OK, Json(user)))
}

pub async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<UserProfile>), AppError> {
    let creat_user = service_user::svc_create_user(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(creat_user)))
}

pub async fn delete_data_user(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<&'static str>, AppError> {
    let delt_user = service_user::svc_delete_user(&pool, id).await?;

    Ok(Json(delt_user))
}
