use axum::{Extension, Json, extract::State, http::StatusCode};

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::{ApiResponse, request::request_user::Decant},
    error::error::AppError,
    service::decant_svc::svc_create_decant,
    state::AppState,
};

pub async fn create_decant(
    State(state): State<AppState>,
    Json(req): Json<Decant>,
    Extension(access): Extension<AccesClaims>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
    let create = svc_create_decant(&state.db, &req, &access).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: (),
            message: Some(create),
        }),
    ))
}
