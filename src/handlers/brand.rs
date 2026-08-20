use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};

use crate::c_auth::refresh_token::AccesClaims;
use crate::dto::ApiResponse;
use crate::dto::request::request_user::CreateBrands;
use crate::error::error::AppError;
use crate::service::brands_svc::svc_create_brands;
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

/*
pub async fn get_all_brands (State(state): State<AppState>) -> Result<(StatusCode, Json<Vec<Brand>>), AppError> {
    let = parfume_svc::svc_get_all_brands().await?;

}
     */
