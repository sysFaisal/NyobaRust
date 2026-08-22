use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::{
        ApiResponse,
        request::batch_req::{CreateBatch, UpdateBatch},
        response::batch_res::BatchResponse,
    },
    error::error::AppError,
    service::batch_svc::{svc_create_batch, svc_get_all_batch, svc_update_batch},
    state::AppState,
};

pub async fn create_batch(
    State(appstate): State<AppState>,
    Extension(access): Extension<AccesClaims>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateBatch>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
    let create = svc_create_batch(&appstate.db, &req, &access, &id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: (),
            message: Some(create),
        }),
    ))
}

pub async fn get_all_batch(
    State(state): State<AppState>,
    Extension(access): Extension<AccesClaims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<BatchResponse>>>), AppError> {
    let res = svc_get_all_batch(&state.db, &access, &id).await?;

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
    Extension(access): Extension<AccesClaims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBatch>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), AppError> {
    let res = svc_update_batch(&state.db, &req, &access, &id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: res,
            message: None,
        }),
    ))
}
