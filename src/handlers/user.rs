use axum::extract::{Path, State};
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, Expiration, SameSite};

use uuid::Uuid;

use crate::AppState;
use crate::c_auth::refresh_token::{AccesClaims, RoleModel};
use crate::dto::ApiResponse;
use crate::dto::request::request_user::{CreateUser, LoginUser};
use crate::dto::response::response_user::{LoginResponse, UserProfile};
use crate::error::error::AppError;
use crate::service::service_user::{self, svc_refresh_token};

/*
Untuk i5-6200U, saya akan coba benchmark begini
Preset	Memory	Time	Parallelism	Perkiraan karakter
A	16 MiB	2	1	ringan
B	19 MiB	2	1	baseline yang saya pilih
C	32 MiB	2	1	sedang
D	32 MiB	3	1	lebih berat
E	64 MiB	2	1	lebih berat lagi

*/

pub async fn get_all_user(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<UserProfile>>>), AppError> {
    let users = service_user::svc_get_all_user(&state.db).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: users,
            message: None,
        }),
    ))
}

pub async fn get_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<UserProfile>>), AppError> {
    let user = service_user::svc_get_user_by_id(&state.db, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: user,
            message: None,
        }),
    ))
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(mut payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<ApiResponse<UserProfile>>), AppError> {
    let new_user = service_user::svc_create_user(&state.dns, &state.db, &mut payload).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: new_user,
            message: Some("Succes Create User".to_string()),
        }),
    ))
}

pub async fn delete_data_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<AccesClaims>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
    if claims.role != RoleModel::Dev as RoleModel {
        return Err(AppError::Forbidden);
    }

    let my_uuid = match Uuid::parse_str(&claims.sub) {
        Ok(val) => val,
        Err(_) => return Err(AppError::BadRequest(None)),
    };

    if my_uuid == id {
        return Err(AppError::Forbidden);
    }

    let delete_user_response = service_user::svc_delete_user(&state.db, id).await?;

    Ok((
        StatusCode::NO_CONTENT,
        Json(ApiResponse {
            data: (),
            message: Some(delete_user_response.to_string()),
        }),
    ))
}

pub async fn login_user(
    State(state): State<AppState>,
    Json(payload): Json<LoginUser>,
) -> Result<(StatusCode, HeaderMap, Json<ApiResponse<LoginResponse>>), AppError> {
    let (refresh_cookie_value, expire_at, jwt) =
        service_user::svc_login_user(&state.db, &payload).await?;

    let cookie = Cookie::build(("refresh_token", refresh_cookie_value))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .expires(Expiration::DateTime(expire_at))
        .build();

    let mut header = HeaderMap::new();
    header.insert(SET_COOKIE, cookie.to_string().parse().unwrap());

    Ok((
        StatusCode::OK,
        header,
        Json(ApiResponse {
            data: LoginResponse {
                access_token: jwt,
                token_type: "Bearer".to_string(),
            },
            message: Some("Success".to_string()),
        }),
    ))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(StatusCode, CookieJar, Json<ApiResponse<LoginResponse>>), AppError> {
    let cookie = jar.get("refresh_token").ok_or(AppError::Unauthorized)?;
    let cookie_value = cookie.value();

    let (family_id, incoming_token) = cookie_value.split_once('.').ok_or(AppError::Unauthorized)?;

    let (new_access_token, new_cookie_value) =
        svc_refresh_token(&state.db, family_id, incoming_token).await?;

    let new_cookie = Cookie::build(("refresh_token", new_cookie_value))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .build();

    let new_jar = jar.add(new_cookie);

    Ok((
        StatusCode::OK,
        new_jar,
        Json(ApiResponse {
            data: LoginResponse {
                access_token: new_access_token,
                token_type: "Bearer".to_string(),
            },
            message: Some("Success".to_string()),
        }),
    ))
}
