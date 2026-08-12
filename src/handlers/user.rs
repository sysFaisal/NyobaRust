use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use sqlx::types::JsonValue::Null;
use uuid::Uuid;

use crate::AppState;
use crate::dto::ApiResponse;
use crate::dto::request::request_user::{CreateUser, LoginUser};
use crate::dto::response::response_user::UserProfile;
use crate::error::error::AppError;
use crate::service::service_user;

/*
Untuk i5-6200U, saya akan coba benchmark begini
Preset	Memory	Time	Parallelism	Perkiraan karakter
A	16 MiB	2	1	ringan
B	19 MiB	2	1	baseline yang saya pilih
C	32 MiB	2	1	sedang
D	32 MiB	3	1	lebih berat
E	64 MiB	2	1	lebih berat lagi

*/

pub async fn get_all_user(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<UserProfile>>>), AppError> {
    let users = service_user::svc_get_all_user(&state.db).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: users,
            message: None,
        }),
    ))
}

pub async fn get_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<UserProfile>>), AppError> {
    let user = service_user::svc_get_user_by_id(&state.db, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: user,
            message: None,
        }),
    ))
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(mut payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<ApiResponse<UserProfile>>), AppError> {
    let new_user = service_user::svc_create_user(&state.dns, &state.db, &mut payload).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: new_user,
            message: Some("Succes Create User".to_string()),
        }),
    ))
}

pub async fn delete_data_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let delete_user_response = service_user::svc_delete_user(&state.db, id).await?;

    Ok(Json(ApiResponse {
        data: (),
        message: Some(delete_user_response.to_string()),
    }))
}

pub async fn login_user(
    State(state): State<AppState>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let result = service_user::svc_login_user(&state.db, &payload).await?;

    Ok(Json(ApiResponse {
        data: "".to_string(),
        message: Some(result),
    }))
}
