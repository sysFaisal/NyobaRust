use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::request::request_user::CreateUser;
use crate::dto::response::response_user::UserProfile;
use crate::error::error::AppError;

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

pub fn validate_trim_len_string(value: &str, min_length: usize) -> bool {
    let str_trimmed = value.trim();
    if str_trimmed.is_empty() {
        return false;
    }
    if str_trimmed.len() < min_length {
        return false;
    }
    true
}

pub async fn svc_create_user(pool: &PgPool, payload: CreateUser) -> Result<UserProfile, AppError> {
    if !validate_trim_len_string(&payload.username, 3) {
        return Err(AppError::BadRequest(None));
    }

    if !validate_trim_len_string(&payload.password, 8) {
        return Err(AppError::BadRequest(None));
    }

    match &payload.email {
        Some(email) => {
            let respon = sqlx::query_as!(
                UserProfile,
                r#"
                INSERT INTO users (username, email, password_hash)
                VALUES ($1, $2, $3)
                RETURNING id, username, email, created_at"#,
                payload.username,
                email,
                payload.password,
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
                payload.password,
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
