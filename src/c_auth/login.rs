use crate::c_auth::refresh_token::{RoleModel, generate_access_token, generate_refresh_token};
use crate::dto::request::user_req::LoginUser;
use crate::error::error::AppError;
use crate::service::validation::verify_password;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

pub async fn get_id_pw_by_username(
    pool: &PgPool,
    username: &String,
) -> Result<(Uuid, String, RoleModel), AppError> {
    if username.trim().is_empty() {
        return Err(AppError::BadRequest(None, Some("get_id_pw_by_username: username kosong".to_string())));
    }

    let user = sqlx::query!(
        r#"SELECT id, password_hash, role as "role: RoleModel" FROM users WHERE username = $1"#,
        username
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound(None, Some("get_id_pw_by_username: user tidak ditemukan".to_string())))?;

    Ok((user.id, user.password_hash, user.role))
}

pub async fn svc_login_user(
    pool: &PgPool,
    payload: &LoginUser,
) -> Result<(String, OffsetDateTime, String), AppError> {
    if payload.username.trim().is_empty() || payload.password.trim().is_empty() {
        return Err(AppError::BadRequest(None, Some("svc_login_user: username atau password kosong".to_string())));
    }

    let (user_id, hashed_password, role) = get_id_pw_by_username(pool, &payload.username).await?;

    match verify_password(&payload.password, &hashed_password) {
        Ok(true) => {}
        Ok(false) => return Err(AppError::Unauthorized(None, Some("svc_login_user: password salah".to_string()))),
        Err(_) => return Err(AppError::InternalServerError(None, Some("svc_login_user: error saat verify password".to_string()))),
    }

    let family_id = Uuid::new_v4();
    let token = generate_refresh_token();

    let chrono_to_time = match OffsetDateTime::from_unix_timestamp(token.expire_at.timestamp()) {
        Ok(time) => time,
        Err(_) => return Err(AppError::InternalServerError(None, Some("svc_login_user: gagal konversi timestamp ke OffsetDateTime".to_string()))),
    };

    let cookie_value = format!("{}.{}", family_id, token.token);

    sqlx::query!(
        r#"
    INSERT INTO refresh_token (
        user_id,
        token_hash,
        expire_at,
        family_id
    )
    VALUES (
        $1,
        $2,
        $3,
        $4
    )
    "#,
        user_id,
        token.token_hash,
        token.expire_at,
        family_id,
    )
    .execute(pool)
    .await?;

    let jwt = generate_access_token(user_id, &role)?;

    Ok((cookie_value, chrono_to_time, jwt))
}
