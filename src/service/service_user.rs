use hickory_resolver::TokioResolver;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::dto::request::request_user::{CreateUser, LoginUser};
use crate::dto::response::response_user::UserProfile;
use crate::error::error::AppError;
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

pub async fn get_password_by_username(
    pool: &PgPool,
    username: &str,
) -> Result<Option<String>, AppError> {
    let password_hash = sqlx::query_scalar!(
        r#"SELECT password_hash FROM users WHERE username = $1"#,
        username
    )
    .fetch_optional(pool)
    .await?;

    Ok(password_hash)
}

pub async fn svc_login_user(pool: &PgPool, payload: &LoginUser) -> Result<String, AppError> {
    if payload.username.trim().is_empty() {
        return Err(AppError::BadRequest(None));
    }

    if payload.password.trim().is_empty() {
        return Err(AppError::BadRequest(None));
    }

    let hashed_password = match get_password_by_username(pool, &payload.username).await? {
        Some(hashed) => hashed,
        None => return Err(AppError::Unauthorized),
    };

    match verify_password(&payload.password, &hashed_password) {
        Ok(true) => {}
        Ok(false) => return Err(AppError::Unauthorized),
        Err(_) => return Err(AppError::InternalServerError(None)),
    }

    Ok("Berhasil".to_string())
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
