use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};

use crate::{
    c_auth::auth_user::AuthUser,
    dto::{
        ApiResponse,
        request::revenue_req::{DailyRevenueQuery, RevenueHistoryQuery},
        response::revenue_res::{
            DailyRevenuePoint, RevenueHistoryPoint, RevenueSummaryResponse,
        },
    },
    error::error::AppError,
    service::revenue_svc::{svc_get_daily_revenue, svc_get_revenue_history, svc_get_revenue_summary},
    state::AppState,
};

pub async fn get_revenue_summary(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<(StatusCode, Json<ApiResponse<RevenueSummaryResponse>>), AppError> {
    let summary = svc_get_revenue_summary(&state.db, user.id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: summary,
            message: Some("Succes".to_string()),
        }),
    ))
}

pub async fn get_daily_revenue(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<DailyRevenueQuery>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<DailyRevenuePoint>>>), AppError> {
    let res = svc_get_daily_revenue(&state.db, user.id, &query).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: res,
            message: Some("Succes".to_string()),
        }),
    ))
}

pub async fn get_revenue_history(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<RevenueHistoryQuery>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<RevenueHistoryPoint>>>), AppError> {
    let res = svc_get_revenue_history(&state.db, user.id, &query).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: res,
            message: Some("Succes".to_string()),
        }),
    ))
}
