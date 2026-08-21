use axum::{Extension, Json, extract::State, http::StatusCode};

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::{ApiResponse, request::request_user::Batch},
    error::error::AppError,
    service::batch_svc::svc_create_batch,
    state::AppState,
};

pub async fn create_batch(
    State(appstate): State<AppState>,
    Extension(access): Extension<AccesClaims>,
    Json(req): Json<Batch>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
    let create = svc_create_batch(&appstate.db, &req, &access).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: (),
            message: Some(create),
        }),
    ))
}
