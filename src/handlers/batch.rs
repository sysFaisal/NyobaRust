use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    c_auth::auth_user::AuthUser,
    dto::{
        ApiResponse,
        request::batch_req::{CreateBatch, UpdateBatch},
        response::batch_res::BatchResponse,
    },
    error::error::AppError,
    service::batch_svc::{svc_create_batch, svc_delete_batch, svc_get_all_batch, svc_update_batch},
    state::AppState,
};

pub async fn create_batch(
    State(appstate): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateBatch>,
) -> Result<(StatusCode, Json<ApiResponse<BatchResponse>>), AppError> {
    let create = svc_create_batch(&appstate.db, &req, user.id, &id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: create,
            message: Some("Created Batch".to_string()),
        }),
    ))
}

pub async fn get_all_batch(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<BatchResponse>>>), AppError> {
    let res = svc_get_all_batch(&state.db, user.id, &id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: res,
            message: Some("Succes".to_string()),
        }),
    ))
}

pub async fn update_batch(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBatch>,
) -> Result<(StatusCode, Json<ApiResponse<BatchResponse>>), AppError> {
    let res = svc_update_batch(&state.db, &req, user.id, &id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: res,
            message: Some("Berhasil".to_string()),
        }),
    ))
}

pub async fn delete_batch(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), AppError> {
    let result = svc_delete_batch(&state.db, user.id, &id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: result,
            message: None,
        }),
    ))
}
