use chrono::{DateTime, Utc};
use hex::encode;
use hickory_resolver::TokioResolver;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::c_auth::refresh_token::{
    RefreshToken, generate_access_token, generate_refresh_token, hash_token_sha256,
};
use crate::dto::request::request_user::{CreateUser, LoginUser};
use crate::dto::response::response_user::UserProfile;
use crate::error::error::AppError;
use crate::handlers::user;
use crate::service::validation::{hash_password, validate_email, verify_password};

pub async fn svc_get_all_user(pool: &PgPool) -> Result<Vec<UserProfile>, AppError> {
    let users = sqlx::query_as!(
        UserProfile,
        "SELECT id, username, email, created_at FROM users"
    )
    .fetch_all(pool)
    .await?;

    Ok(users)
}

pub async fn svc_get_user_by_id(
    pool: &PgPool,
    target_id: &Uuid,
) -> Result<Option<UserProfile>, AppError> {
    let users = sqlx::query_as!(
        UserProfile,
        r#"
        SELECT id, username, email, created_at
        FROM users
        WHERE id = $1
        "#,
        target_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(users)
}

pub async fn get_id_pw_by_username(
    pool: &PgPool,
    username: &String,
) -> Result<(Uuid, String), AppError> {
    if username.trim().is_empty() {
        return Err(AppError::BadRequest(None));
    }

    let user = sqlx::query!(
        r#"SELECT id, password_hash FROM users WHERE username = $1"#,
        username
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    Ok((user.id, user.password_hash))
}
pub async fn svc_login_user(
    pool: &PgPool,
    payload: &LoginUser,
) -> Result<(String, OffsetDateTime, String), AppError> {
    if payload.username.trim().is_empty() || payload.password.trim().is_empty() {
        return Err(AppError::BadRequest(None));
    }

    let (user_id, hashed_password) = get_id_pw_by_username(pool, &payload.username).await?;

    match verify_password(&payload.password, &hashed_password) {
        Ok(true) => {}
        Ok(false) => return Err(AppError::Unauthorized),
        Err(_) => return Err(AppError::InternalServerError(None)),
    }

    let family_id = Uuid::new_v4();
    let token = generate_refresh_token();

    let chrono_to_time = match OffsetDateTime::from_unix_timestamp(token.expire_at.timestamp()) {
        Ok(time) => time,
        Err(_) => return Err(AppError::InternalServerError(None)),
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
        family_id
    )
    .execute(pool)
    .await?;

    let jwt = generate_access_token(user_id)?;

    Ok((cookie_value, chrono_to_time, jwt))
}

pub async fn svc_create_user(
    dns: &TokioResolver,
    pool: &PgPool,
    payload: &mut CreateUser,
) -> Result<UserProfile, AppError> {
    payload.username = payload.username.trim().to_string();

    if payload.password.trim().is_empty() {
        return Err(AppError::BadRequest(None));
    }

    match payload.validate() {
        Ok(_) => {}
        Err(e) => return Err(AppError::BadRequest(Some(e.to_string()))),
    };

    if let Some(email) = &payload.email {
        if !validate_email(dns, email.as_str()).await {
            return Err(AppError::BadRequest(None));
        }
    }

    let password_hash = match hash_password(&payload.password.as_str()) {
        Ok(hash) => hash,
        Err(_) => return Err(AppError::BadRequest(None)),
    };

    match &payload.email {
        Some(email) => {
            let respon = sqlx::query_as!(
                UserProfile,
                r#"
                INSERT INTO users (username, email, password_hash)
                VALUES ($1, $2, $3)
                RETURNING id, username, email, created_at"#,
                payload.username,
                email.as_str(),
                password_hash,
            )
            .fetch_one(pool)
            .await?;

            Ok(respon)
        }
        None => {
            let respon = sqlx::query_as!(
                UserProfile,
                r#"INSERT INTO users (username, password_hash)
                VALUES ($1, $2)
                RETURNING id, username, email, created_at"#,
                payload.username,
                password_hash,
            )
            .fetch_one(pool)
            .await?;

            Ok(respon)
        }
    }
}

//fetch one untuk mengambil data sekali T
//fetch all untuk mengambil data banyak Vec<T>
//fetch optional untuk mengambil data dengan Option<T>
//execute untuk query tanpa mengambil row QueryResult

//row kosong
//fetch one masuk ke sqlx::error::rownotound
//fetch all jadi ok(vec!<t>)
//fetch option jadi ok(option => (none))
//execute jadi queryresult diceknya pakai Ok(queryresult)=>affected_row)

//query hasilnya tidak dipetakan ke struct
//query_as dipetakan ke struct
pub async fn svc_delete_user(pool: &PgPool, id: Uuid) -> Result<&'static str, AppError> {
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok("Success Delete")
}

pub async fn svc_refresh_token(
    pool: &PgPool,
    family_id: &str,
    incoming_token: &str,
) -> Result<(String, String), AppError> {
    let family_uuid = Uuid::parse_str(family_id).map_err(|_| AppError::Unauthorized)?;
    let incoming_hash = hex::encode(hash_token_sha256(incoming_token.as_bytes()));

    let mut transaction = pool.begin().await?;

    let record = sqlx::query!(
        r#"
        SELECT user_id, token_hash, expire_at
        FROM refresh_token
        WHERE family_id = $1
        "#,
        family_uuid
    )
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if record.token_hash != incoming_hash {
        sqlx::query!(
            r#"DELETE FROM refresh_token WHERE family_id = $1"#,
            family_uuid
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        return Err(AppError::Unauthorized);
    }

    if record.expire_at < Utc::now() {
        sqlx::query!(
            r#"DELETE FROM refresh_token WHERE family_id = $1"#,
            family_uuid
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        return Err(AppError::Unauthorized);
    }

    let new_access_token = generate_access_token(record.user_id)?;
    let new_refresh_token = generate_refresh_token();
    let new_cookie_value = format!("{}.{}", family_uuid, new_refresh_token.token);
    let new_expire_at = Utc::now() + chrono::Duration::days(7);

    sqlx::query!(
        r#"
        UPDATE refresh_token
        SET token_hash = $1,
            expire_at = $2
        WHERE family_id = $3
        "#,
        new_refresh_token.token_hash,
        new_expire_at,
        family_uuid
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok((new_access_token, new_cookie_value))
}
