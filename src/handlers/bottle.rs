use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    c_auth::refresh_token::AccesClaims, dto::{
        ApiResponse,
        request::botol_req::{CreateBottle, UpdateBottle},
        response::botol_res::BotolResponse,
    }, error::error::AppError, service::bottle_svc::{
        svc_create_bottle, svc_delete_bottle, svc_get_all_bottle, svc_get_bottle, svc_update_bottle,
    }, state::AppState,
};

pub async fn create_bottle(
    State(state): State<AppState>,
    Extension(access): Extension<AccesClaims>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateBottle>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
    let create = svc_create_bottle(&state.db, &req, &access, &id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: (),
            message: Some(create),
        }),
    ))

}

pub async fn get_bottle(
    State(state): State<AppState>,
    Extension(access): Extension<AccesClaims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<BotolResponse>>), AppError> {
    let bottle = svc_get_bottle(&state.db, &access, &id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: bottle,
            message: None,
        }),
    ))
}

pub async fn update_bottle(
    State(state): State<AppState>,
    Extension(access): Extension<AccesClaims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBottle>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), AppError> {
    let result = svc_update_bottle(&state.db, &access, &id, &req).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: result,
            message: None,
        }),
    ))
}

pub async fn delete_bottle(
    State(state): State<AppState>,
    Extension(access): Extension<AccesClaims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), AppError> {
    let result = svc_delete_bottle(&state.db, &access, &id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: result,
            message: None,
        }),
    ))
}

pub async fn get_all_bottle(
    State(state): State<AppState>,
    Extension(access): Extension<AccesClaims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<BotolResponse>>>), AppError> {
    let res = svc_get_all_bottle(&state.db, &access, &id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: res,
            message: Some("Succes".to_string()),
        }),
    ))
}
