use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    c_auth::auth_user::AuthUser, dto::{
        ApiResponse, PaginatedResponse, PaginationMeta,
        request::{
            parfume_req::{CreateParfume, RankingQuery, UpdateParfume},
            revenue_req::RevenueHistoryQuery,
        },
        response::{
            parfume_res::{ParfumeRankingPoint, ParfumeResponse},
            revenue_res::RevenueHistoryPoint,
        },
    }, error::error::AppError, service::parfume_svc::{svc_create_parfume, svc_delete_parfume, svc_get_all_parfume, svc_get_all_parfume_uni, svc_get_parfume_by_id, svc_get_parfume_history, svc_get_parfume_ranking, svc_update_parfume, RANKING_DEFAULT_PER_PAGE}, state::AppState,
};

pub async fn create_parfum(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateParfume>,
) -> Result<(StatusCode, Json<ApiResponse<ParfumeResponse>>), AppError> {
    let create = svc_create_parfume(&state.db, &req, user.id, user.role).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: create,
            message: Some("Parfume created successfully".to_string()),
        }),
    ))
}

pub async fn get_all_parfume(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<ParfumeResponse>>>), AppError> {
    let res = svc_get_all_parfume(&state.db, user.id, &id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: res,
            message: Some("Succes".to_string()),
        }),
    ))
}

pub async fn get_all_parfume_uni(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<(StatusCode, Json<ApiResponse<Vec<ParfumeResponse>>>), AppError> {
    let res = svc_get_all_parfume_uni(&state.db, user.id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: res,
            message: Some("Succes".to_string()),
        }),
    ))
}

pub async fn get_parfume_by_id(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<ParfumeResponse>>), AppError> {
    let parfume = svc_get_parfume_by_id(&state.db, user.id, &id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: parfume,
            message: None,
        }),
    ))
}

pub async fn update_parfume(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateParfume>,
) -> Result<(StatusCode, Json<ApiResponse<ParfumeResponse>>), AppError> {
    let parfume = svc_update_parfume(&state.db, &req, user.id, &id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: parfume,
            message: Some("Berhasil".to_string()),
        }),
    ))
}

pub async fn delete_parfume(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), AppError> {
    let result = svc_delete_parfume(&state.db, user.id, &id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: result,
            message: None,
        }),
    ))
}

pub async fn get_parfume_ranking(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<RankingQuery>,
) -> Result<(StatusCode, Json<PaginatedResponse<ParfumeRankingPoint>>), AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(RANKING_DEFAULT_PER_PAGE);
    let (res, total_items) = svc_get_parfume_ranking(&state.db, user.id, &query).await?;

    let total_pages = (total_items + per_page - 1) / per_page;

    Ok((
        StatusCode::OK,
        Json(PaginatedResponse {
            data: res,
            pagination: PaginationMeta {
                page,
                per_page,
                total_items,
                total_pages,
            },
            message: Some("Succes".to_string()),
        }),
    ))
}

pub async fn get_parfume_history(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Query(query): Query<RevenueHistoryQuery>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<RevenueHistoryPoint>>>), AppError> {
    let res = svc_get_parfume_history(&state.db, user.id, &id, &query).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: res,
            message: Some("Succes".to_string()),
        }),
    ))
}
