use crate::c_auth::refresh_token::RoleModel;
use crate::c_auth::refresh_token::{
    AccesClaims, generate_access_token, generate_refresh_token, hash_token_sha256,
};
use crate::dto::request::user_req::{CreateUser, UpdateUser};
use crate::dto::response::user_res::UserProfile;
use crate::env::get_jwt_key;
use crate::error::error::AppError;
use crate::service::validation::{hash_password, validate_email};
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use chrono::Utc;
use hickory_resolver::TokioResolver;
use jsonwebtoken::{DecodingKey, decode};
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;
use validator::Validate;

pub async fn svc_get_all_user(pool: &PgPool) -> Result<Vec<UserProfile>, AppError> {
    let users = sqlx::query_as!(
        UserProfile,
        r#"
            SELECT
                id,
                username,
                email,
                created_at
            FROM users
        "#
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

pub async fn svc_create_user(
    dns: &TokioResolver,
    pool: &PgPool,
    payload: &mut CreateUser,
) -> Result<UserProfile, AppError> {
    payload.username = payload.username.trim().to_string();

    if payload.password.trim().is_empty() {
        return Err(AppError::BadRequest(None, Some("svc_create_user: password kosong".to_string())));
    }

    match payload.validate() {
        Ok(_) => {}
        Err(e) => return Err(AppError::BadRequest(Some(e.to_string()), Some("svc_create_user: validasi input gagal".to_string()))),
    };

    if let Some(email) = &payload.email {
        if !validate_email(dns, email.as_str()).await {
            return Err(AppError::BadRequest(None, Some("svc_create_user: email tidak valid".to_string())));
        }
    }

    let password_hash = match hash_password(&payload.password.as_str()) {
        Ok(hash) => hash,
        Err(_) => return Err(AppError::BadRequest(None, Some("svc_create_user: hash password gagal".to_string()))),
    };

    let seller = RoleModel::Seller;
    match &payload.email {
        Some(email) => {
            let respon = sqlx::query_as!(
                UserProfile,
                r#"
                INSERT INTO users (username, email, password_hash, role )
                VALUES ($1, $2, $3, $4)
                RETURNING id, username, email, created_at"#,
                payload.username,
                email.as_str(),
                password_hash,
                RoleModel::Seller as RoleModel
            )
            .fetch_one(pool)
            .await?;

            Ok(respon)
        }
        None => {
            let respon = sqlx::query_as!(
                UserProfile,
                r#"INSERT INTO users (username, password_hash, role)
                VALUES ($1, $2, $3)
                RETURNING id, username, email, created_at"#,
                payload.username,
                password_hash,
                RoleModel::Seller as RoleModel
            )
            .fetch_one(pool)
            .await?;

            Ok(respon)
        }
    }
}

pub async fn svc_update_user(
    dns: &TokioResolver,
    pool: &PgPool,
    id: &Uuid,
    payload: &UpdateUser,
) -> Result<UserProfile, AppError> {
    if payload.username.is_none() && payload.email.is_none() && payload.password.is_none() {
        return Err(AppError::BadRequest(
            Some("No field to update".to_string()),
            Some("svc_update_user: tidak ada field yang dikirim client".to_string()),
        ));
    }

    payload
        .validate()
        .map_err(|e| AppError::BadRequest(Some(e.to_string()), Some("svc_update_user: validasi input gagal".to_string())))?;

    let current = sqlx::query!(
        r#"SELECT username, email, password_hash FROM users WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound(None, Some("svc_update_user: user tidak ditemukan".to_string())))?;

    let username = match &payload.username {
        Some(username) => {
            let username = username.trim().to_string();
            if username.is_empty() {
                return Err(AppError::BadRequest(None, Some("svc_update_user: username kosong".to_string())));
            }
            username
        }
        None => current.username,
    };

    let email = match &payload.email {
        Some(Some(email)) => {
            if !validate_email(dns, email.as_str()).await {
                return Err(AppError::BadRequest(None, Some("svc_update_user: email tidak valid".to_string())));
            }
            Some(email.as_str().to_string())
        }
        Some(None) => None,
        None => current.email,
    };

    let password_hash = match &payload.password {
        Some(password) => {
            let password = password.trim().to_string();
            if password.is_empty() {
                return Err(AppError::BadRequest(None, Some("svc_update_user: password kosong".to_string())));
            }
            hash_password(&password).map_err(|_| AppError::BadRequest(None, Some("svc_update_user: hash password gagal".to_string())))?
        }
        None => current.password_hash,
    };

    let row = sqlx::query!(
        r#"
        UPDATE users
        SET username = $1, email = $2, password_hash = $3
        WHERE id = $4
        RETURNING id, username, email, created_at
        "#,
        username,
        email,
        password_hash,
        id
    )
    .fetch_one(pool)
    .await?;

    Ok(UserProfile {
        id: row.id,
        username: row.username,
        email: row.email,
        created_at: row.created_at,
    })
}

pub async fn svc_delete_user(pool: &PgPool, id: Uuid) -> Result<&'static str, AppError> {
    let mut tx = pool.begin().await?;

    let owned_brands = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM brands WHERE owner_id = $1"#,
        id
    )
    .fetch_one(&mut *tx)
    .await?;

    if owned_brands > 0 {
        return Err(AppError::Conflict(
            Some(format!("user masih memiliki {owned_brands} brand, hapus atau pindahkan brand terlebih dahulu")),
            Some("svc_delete_user: user masih direferensikan oleh brands".to_string()),
        ));
    }

    let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(None, Some("svc_delete_user: user tidak ditemukan".to_string())));
    }

    tx.commit().await?;
    Ok("Success Delete")
}

pub async fn svc_refresh_token(
    pool: &PgPool,
    family_id: &str,
    incoming_token: &str,
) -> Result<(String, String), AppError> {
    let family_uuid = Uuid::parse_str(family_id).map_err(|_| {
        warn!(family_id = %family_id, "invalid family id sent during refresh");
        AppError::Unauthorized(None, Some("svc_refresh_token: family_id tidak valid".to_string()))
    })?;
    let incoming_hash = hex::encode(hash_token_sha256(incoming_token.as_bytes()));

    let mut transaction = pool.begin().await?;

    let record = sqlx::query!(
        r#"
        SELECT rt.user_id, rt.token_hash, rt.expire_at, u.role AS "role: RoleModel"
        FROM refresh_token rt
        JOIN users u ON u.id = rt.user_id
        WHERE rt.family_id = $1
        "#,
        family_uuid
    )
    .fetch_optional(&mut *transaction)
    .await?;

    let record = match record {
        Some(value) => value,
        None => {
            warn!(family_id = %family_uuid, "refresh token family not found during refresh");
            return Err(AppError::Unauthorized(None, Some("svc_refresh_token: token family tidak ditemukan".to_string())));
        }
    };

    if record.token_hash != incoming_hash {
        warn!(family_id = %family_uuid, stored_hash = %record.token_hash, incoming_hash = %incoming_hash, "refresh token reuse detected, deleting token family");
        sqlx::query!(
            r#"DELETE FROM refresh_token WHERE family_id = $1"#,
            family_uuid
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        return Err(AppError::Unauthorized(None, Some("svc_refresh_token: refresh token reuse terdeteksi".to_string())));
    }

    if record.expire_at < Utc::now() {
        warn!(family_id = %family_uuid, expire_at = ?record.expire_at, "refresh token expired, deleting token family");
        sqlx::query!(
            r#"DELETE FROM refresh_token WHERE family_id = $1"#,
            family_uuid
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        return Err(AppError::Unauthorized(None, Some("svc_refresh_token: refresh token sudah expired".to_string())));
    }

    let role = record.role;

    info!(family_id = %family_uuid, user_id = %record.user_id, "refresh token validation successful, starting rotation");

    let new_access_token = generate_access_token(record.user_id, &role)?;
    let new_refresh_token = generate_refresh_token();
    let new_cookie_value = format!("{}.{}", family_uuid, new_refresh_token.token);
    let new_expire_at = Utc::now() + chrono::Duration::days(7);

    let update_result = sqlx::query!(
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
    .await;

    match update_result {
        Ok(_) => {
            info!(family_id = %family_uuid, new_expire_at = ?new_expire_at, "refresh token rotated and database updated");
        }
        Err(err) => {
            error!(family_id = %family_uuid, error = ?err, "failed to rotate refresh token in database");
            return Err(err.into());
        }
    }

    transaction.commit().await?;

    Ok((new_access_token, new_cookie_value))
}

pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, AppError> {
    let auth = match req.headers().get(axum::http::header::AUTHORIZATION) {
        Some(val) => val,
        None => return Err(AppError::BadRequest(
            Some("Test".to_string()),
            Some("auth_middleware: header Authorization tidak ada".to_string()),
        )),
    };

    let auth_str = match auth.to_str() {
        Ok(token) => token,
        Err(e) => {
            return Err(AppError::InternalServerError(
                None,
                Some("auth_middleware: gagal parse header Authorization ke &str".to_string()),
            ));
        }
    };

    let jwt = match auth_str.strip_prefix("Bearer ") {
        Some(val) => val,
        None => return Err(AppError::BadRequest(None, Some("auth_middleware: header tidak mengandung Bearer token".to_string()))),
    };

    let secret = match get_jwt_key() {
        Ok(val) => val,
        Err(e) => return Err(AppError::InternalServerError(None, Some(e.to_string()))),
    };

    let claims = match decode::<AccesClaims>(
        jwt,
        &DecodingKey::from_secret(secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    ) {
        Ok(val) => val.claims,
        Err(_) => return Err(AppError::BadRequest(
            Some("dwdw".to_string()),
            Some("auth_middleware: gagal decode JWT access token".to_string()),
        )),
    };

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}
