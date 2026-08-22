use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    c_auth::refresh_token::AccesClaims, dto::{
        ApiResponse, request::parfume_req::CreateParfume, response::parfume_res::ParfumeResponse,
    }, error::error::AppError, service::parfume_svc::{svc_create_parfume, svc_get_all_parfume, svc_get_all_parfume_uni, svc_get_parfume_by_id}, state::AppState,
};

pub async fn create_parfum(
    State(state): State<AppState>,
    Extension(access): Extension<AccesClaims>,
    Json(req): Json<CreateParfume>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
    let create = svc_create_parfume(&state.db, &req, &access).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: (),
            message: Some(create),
        }),
    ))
}

pub async fn get_all_parfume(
    State(state): State<AppState>,
    Extension(access): Extension<AccesClaims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<ParfumeResponse>>>), AppError> {
    let res = svc_get_all_parfume(&state.db, &access, &id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: res,
            message: Some("Succes".to_string()),
        }),
    ))
}

pub async fn get_all_parfume_uni(
    State(state): State<AppState>,
    Extension(access): Extension<AccesClaims>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<ParfumeResponse>>>), AppError> {
    let res = svc_get_all_parfume_uni(&state.db, &access).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: res,
            message: Some("Succes".to_string()),
        }),
    ))
}

pub async fn get_parfume_by_id(
    State(state): State<AppState>,
    Extension(access): Extension<AccesClaims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<ParfumeResponse>>), AppError> {
    let parfume = svc_get_parfume_by_id(&state.db, &access, &id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: parfume,
            message: None,
        }),
    ))
}
