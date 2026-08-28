use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::{
        ApiResponse, PaginatedResponse, PaginationMeta,
        request::order_req::{CreateOrder, OrderPageQuery, UpdateOrderStatus},
        response::order_mod::OrderResponse,
    },
    error::error::AppError,
    service::order_svc::{
        ORDER_PER_PAGE, svc_create_order, svc_get_all_order, svc_update_order_status,
    },
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

pub async fn update_order(
    State(state): State<AppState>,
    Extension(access): Extension<AccesClaims>,
    Path(id): Path<uuid::Uuid>,
    Json(req): Json<UpdateOrderStatus>,
) -> Result<(StatusCode, Json<ApiResponse<OrderResponse>>), AppError> {
    let updated = svc_update_order_status(&state.db, &id, req.status, &access).await?;
    let msg = match updated.status {
        crate::dto::response::order_mod::OrderStatus::Failed => {
            "order diubah ke failed, stok dikembalikan"
        }
        crate::dto::response::order_mod::OrderStatus::Refund => {
            "order di-refund, stok dikembalikan"
        }
        _ => "berhasil",
    };
    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: updated,
            message: Some(msg.to_string()),
        }),
    ))
}
