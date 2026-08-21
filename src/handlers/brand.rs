use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use uuid::Uuid;

use crate::c_auth::refresh_token::AccesClaims;
use crate::dto::ApiResponse;
use crate::dto::request::brand_req::{CreateBrands, UpdateBrands};
use crate::dto::response::brand_res::Brand;
use crate::error::error::AppError;
use crate::service::brands_svc::{
    svc_create_brands, svc_get_all_brands, svc_get_brands_by_id, svc_update_brands,
};
use crate::state::AppState;

pub async fn create_brands(
    State(state): State<AppState>,
    Extension(access): Extension<AccesClaims>,
    Json(req): Json<CreateBrands>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
    let create = svc_create_brands(&state.db, &req, &access).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: (),
            message: Some(create),
        }),
    ))
}

pub async fn get_all_brands(
    State(state): State<AppState>,
    Extension(access): Extension<AccesClaims>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<Brand>>>), AppError> {
    let res = svc_get_all_brands(&state.db, &access).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: res,
            message: Some("Succes".to_string()),
        }),
    ))
}

pub async fn update_brands(
    State(state): State<AppState>,
    Extension(access): Extension<AccesClaims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBrands>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), AppError> {
    let res = svc_update_brands(&state.db, &req, &access, &id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: res,
            message: None,
        }),
    ))
}

pub async fn get_brands_by_id(
    State(state): State<AppState>,
    Extension(access): Extension<AccesClaims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<Brand>>), AppError> {
    let res = svc_get_brands_by_id(&state.db, &access, &id)
        .await?
        .ok_or(AppError::NotFound(
            None,
            Some("get_brands_by_id: brand tidak ditemukan".to_string()),
        ))?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: res,
            message: None,
        }),
    ))
}

/*
pub async fn get_all_brands (State(state): State<AppState>) -> Result<(StatusCode, Json<Vec<Brand>>), AppError> {
    let = parfume_svc::svc_get_all_brands().await?;

}
     */
