use axum::extract::{Path, State};
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, Expiration, SameSite};

use uuid::Uuid;

use crate::AppState;
use crate::c_auth::auth_user::AuthUser;
use crate::c_auth::login::svc_login_user;
use crate::c_auth::refresh_token::RoleModel;
use crate::dto::ApiResponse;
use crate::dto::request::user_req::{CreateUser, LoginUser, UpdateUser};
use crate::dto::response::user_res::{LoginResponse, UserProfile};
use crate::env::{is_cookie_secure, get_refresh_token_expiry};
use time::OffsetDateTime;
use crate::error::error::AppError;
use crate::service::user_svc::{self, svc_refresh_token, svc_logout};

fn build_refresh_cookie(value: String, expire_at: OffsetDateTime) -> Cookie<'static> {
    Cookie::build(("refresh_token", value))
        .http_only(true)
        .secure(is_cookie_secure())
        .same_site(SameSite::Lax)
        .path("/")
        .expires(Expiration::DateTime(expire_at))
        .build()
}

fn build_remove_cookie() -> Cookie<'static> {
    Cookie::build(("refresh_token", ""))
        .http_only(true)
        .secure(is_cookie_secure())
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(time::Duration::seconds(-1))
        .build()
}

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
    user: AuthUser,
) -> Result<(StatusCode, Json<ApiResponse<Vec<UserProfile>>>), AppError> {
    if user.role != RoleModel::Dev {
        return Err(AppError::Forbidden(None, Some("get_all_user: hanya role Dev yang boleh akses".to_string())));
    };

    let users = user_svc::svc_get_all_user(&state.db).await?;

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
    let user = user_svc::svc_get_user_by_id(&state.db, &id)
        .await?
        .ok_or(AppError::NotFound(None, Some("get_user_by_id: user tidak ditemukan".to_string())))?;

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
    let new_user = user_svc::svc_create_user(&state.dns, &state.db, &mut payload).await?;
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
    user: AuthUser,
) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
    if user.role != RoleModel::Dev {
        return Err(AppError::Forbidden(None, Some("delete_data_user: hanya role Dev yang boleh akses".to_string())));
    }

    if user.id == id {
        return Err(AppError::Forbidden(None, Some("delete_data_user: tidak boleh hapus akun sendiri".to_string())));
    }

    let delete_user_response = user_svc::svc_delete_user(&state.db, id).await?;

    Ok((
        StatusCode::NO_CONTENT,
        Json(ApiResponse {
            data: (),
            message: Some(delete_user_response.to_string()),
        }),
    ))
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    user: AuthUser,
    Json(payload): Json<UpdateUser>,
) -> Result<(StatusCode, Json<ApiResponse<UserProfile>>), AppError> {
    if user.role != RoleModel::Dev && user.id != id {
        return Err(AppError::Forbidden(None, Some("update_user: role bukan Dev dan id tidak cocok".to_string())));
    }

    let user = user_svc::svc_update_user(&state.dns, &state.db, &id, &payload).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            data: user,
            message: Some("Success Update".to_string()),
        }),
    ))
}

pub async fn login_user(
    State(state): State<AppState>,
    Json(payload): Json<LoginUser>,
) -> Result<(StatusCode, HeaderMap, Json<ApiResponse<LoginResponse>>), AppError> {
    let (refresh_cookie_value, expire_at, jwt) = svc_login_user(&state.db, &payload).await?;

    let new_cookie = build_refresh_cookie(refresh_cookie_value, expire_at);
    let remove_cookie = build_remove_cookie();

    let mut header = HeaderMap::new();
    header.append(SET_COOKIE, remove_cookie.to_string().parse().unwrap());
    header.append(SET_COOKIE, new_cookie.to_string().parse().unwrap());

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
) -> Result<(StatusCode, HeaderMap, Json<ApiResponse<LoginResponse>>), AppError> {
    let cookie = jar.get("refresh_token").ok_or(AppError::BadRequest(
        None,
        Some("refresh_token: cookie refresh_token tidak ada di request".to_string()),
    ))?;
    let cookie_value = cookie.value();

    let (family_id, incoming_token) = cookie_value.split_once('.').ok_or(AppError::Unauthorized(None, Some("refresh_token: cookie refresh_token format tidak valid".to_string())))?;

    let (new_access_token, new_cookie_value) =
        svc_refresh_token(&state.db, family_id, incoming_token).await?;

    let expiry_days = get_refresh_token_expiry();
    let new_cookie = build_refresh_cookie(
        new_cookie_value,
        OffsetDateTime::now_utc() + time::Duration::days(expiry_days),
    );
    let remove_cookie = build_remove_cookie();

    let mut header = HeaderMap::new();
    header.append(SET_COOKIE, remove_cookie.to_string().parse().unwrap());
    header.append(SET_COOKIE, new_cookie.to_string().parse().unwrap());

    Ok((
        StatusCode::OK,
        header,
        Json(ApiResponse {
            data: LoginResponse {
                access_token: new_access_token,
                token_type: "Bearer".to_string(),
            },
            message: Some("Success".to_string()),
        }),
    ))
}

pub async fn logout_user(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(StatusCode, HeaderMap, Json<ApiResponse<()>>), AppError> {
    if let Some(cookie) = jar.get("refresh_token") {
        let cookie_value = cookie.value();
        if let Some((family_id, _)) = cookie_value.split_once('.') {
            let _ = svc_logout(&state.db, family_id).await;
        }
    }

    let expired_cookie = build_remove_cookie();

    let mut header = HeaderMap::new();
    header.insert(SET_COOKIE, expired_cookie.to_string().parse().unwrap());

    Ok((
        StatusCode::OK,
        header,
        Json(ApiResponse {
            data: (),
            message: Some("Logged out".to_string()),
        }),
    ))
}
