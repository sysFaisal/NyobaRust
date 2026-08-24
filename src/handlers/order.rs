use axum::{Extension, Json, extract::State, http::StatusCode};

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::{
        ApiResponse,
        request::order_req::CreateOrder,
        response::order_mod::OrderResponse,
    },
    error::error::AppError,
    service::order_svc::{svc_create_order, svc_get_all_order},
    state::AppState,
};

pub async fn create_order(
    State(state): State<AppState>,
    Extension(access): Extension<AccesClaims>,
    Json(req): Json<CreateOrder>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
    let create = svc_create_order(&state.db, &req, &access).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: (),
            message: Some(create),
        }),
    ))
}

pub async fn get_all_order(
    State(state): State<AppState>,
    Extension(access): Extension<AccesClaims>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<OrderResponse>>>), AppError> {
    let res = svc_get_all_order(&state.db, &access).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: res,
            message: Some("Succes".to_string()),
        }),
    ))
}
