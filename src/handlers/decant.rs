use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    c_auth::auth_user::AuthUser, dto::{
        ApiResponse, request::decant_req::{CreateDecant, UpdateDecant}, response::decant_res::DecantResponse,
    }, error::error::AppError, service::decant_svc::{svc_create_decant, svc_delete_decant, svc_get_all_decant, svc_update_decant}, state::AppState,
};

pub async fn create_decant(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    user: AuthUser,
    Json(req): Json<CreateDecant>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {

    let create = svc_create_decant(&state.db, &req, user.id, &id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: (),
            message: Some(create),
        }),
    ))
}

pub async fn get_all_decant(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<DecantResponse>>>), AppError> {
    let res = svc_get_all_decant(&state.db, user.id, &id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: res,
            message: Some("Succes".to_string()),
        }),
    ))
}

pub async fn update_decant(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateDecant>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), AppError> {
    let res = svc_update_decant(&state.db, &req, user.id, &id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: res,
            message: None,
        }),
    ))
}

pub async fn delete_decant(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), AppError> {
    let result = svc_delete_decant(&state.db, user.id, &id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: result,
            message: None,
        }),
    ))
}
