use axum::{Extension, Json, extract::{Path, State}, http::StatusCode};
use uuid::Uuid;

use crate::{c_auth::refresh_token::AccesClaims, dto::{ApiResponse, request::botol_req::CreateBottle}, error::error::AppError, service::bottle_svc::svc_create_bottle, state::AppState};



pub async fn create_bottle(State(state): State<AppState>, Extension(access): Extension<AccesClaims>, Path(id): Path<Uuid>, Json(req): Json<CreateBottle>) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
    let create = svc_create_bottle(&state.db, &req, &access, &id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: (),
            message: Some(create),
        }),
    ))

}