use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
};

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::{
        ApiResponse, PaginatedResponse, PaginationMeta,
        request::order_req::{CreateOrder, OrderPageQuery},
        response::order_mod::OrderResponse,
    },
    error::error::AppError,
    service::order_svc::{ORDER_PER_PAGE, svc_create_order, svc_get_all_order},
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
    Query(query): Query<OrderPageQuery>,
) -> Result<(StatusCode, Json<PaginatedResponse<OrderResponse>>), AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let (res, total_items) = svc_get_all_order(&state.db, &access, page, ORDER_PER_PAGE).await?;

    let total_pages = (total_items + ORDER_PER_PAGE - 1) / ORDER_PER_PAGE;

    Ok((
        StatusCode::OK,
        Json(PaginatedResponse {
            data: res,
            pagination: PaginationMeta {
                page,
                per_page: ORDER_PER_PAGE,
                total_items,
                total_pages,
            },
            message: Some("Succes".to_string()),
        }),
    ))
}
