use axum::{Extension, Json, extract::State, http::StatusCode};

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::{ApiResponse, request::parfume_req::CreateParfume},
    error::error::AppError,
    service::parfume_svc::svc_create_parfume,
    state::AppState,
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
