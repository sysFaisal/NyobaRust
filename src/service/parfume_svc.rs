use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::{request::parfume_req::CreateParfume, response::parfume_res::ParfumeResponse},
    error::error::AppError,
};

pub fn validate_string(value: &str, trimmed: bool, min_length: usize) -> bool {
    let value = if trimmed { value.trim() } else { value };

    value.len() >= min_length
}

pub async fn svc_get_all_parfume(
    pool: &PgPool,
    access: &AccesClaims,
    id: &Uuid,
) -> Result<Vec<ParfumeResponse>, AppError> {
    let uuid = match Uuid::parse_str(&access.sub) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_get_all_parfume: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    let res = sqlx::query_as!(
        ParfumeResponse,
        r#"
        SELECT
            p.id,
            p.brands_id,
            b.name AS "brands_name!",
            p.name,
            p.concentration,
            p.description
        FROM parfume p
        JOIN brands b
            ON p.brands_id = b.id
        WHERE p.brands_id = $1
        AND b.owner_id = $2
        "#,
        id,
        uuid
    )
    .fetch_all(pool)
    .await?;

    Ok(res)
}

pub async fn svc_create_parfume(
    pool: &PgPool,
    req: &CreateParfume,
    access: &AccesClaims,
) -> Result<String, AppError> {
    if !validate_string(&req.name, true, 3) {
        return Err(AppError::BadRequest(
            None,
            Some("svc_create_parfume: nama parfume kurang dari 3 karakter".to_string()),
        ));
    }

    let concentration = match &req.concrentration {
        Some(val) => {
            if !validate_string(&val, true, 3) {
                return Err(AppError::BadRequest(
                    None,
                    Some("svc_create_parfume: concentration kurang dari 3 karakter".to_string()),
                ));
            }

            Some(val.trim().to_string())
        }

        None => None,
    };

    let desc = match &req.description {
        Some(val) => {
            if !validate_string(&val, true, 3) {
                return Err(AppError::BadRequest(
                    None,
                    Some("svc_create_parfume: description kurang dari 3 karakter".to_string()),
                ));
            }

            Some(val.trim().to_string())
        }

        None => None,
    };

    let uuid = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_create_parfume: gagal parse UUID dari claims".to_string()),
            ));
        }
    };
    let result = sqlx::query!(
        r#"
    INSERT INTO parfume (
        brands_id,
        name,
        concentration,
        description
    )
    SELECT
        b.id,
        $3,
        $4,
        $5
    FROM brands b
    WHERE b.id = $1
      AND b.owner_id = $2
    "#,
        req.brands_id,
        uuid,
        req.name.trim(),
        concentration,
        desc,
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Forbidden(
            None,
            Some("svc_create_parfume: brands_id bukan milik user ini".to_string()),
        ));
    }

    Ok("Parfume created successfully".to_string())
}

pub async fn svc_get_all_parfume_uni(
    pool: &PgPool,
    access: &AccesClaims,
) -> Result<Vec<ParfumeResponse>, AppError> {
    let uuid = match Uuid::parse_str(&access.sub) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_get_all_parfume_uni: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    let res = sqlx::query_as!(
        ParfumeResponse,
        r#"
        SELECT
            p.id,
            p.brands_id,
            b.name AS "brands_name!",
            p.name,
            p.concentration,
            p.description
        FROM parfume p
        JOIN brands b
            ON p.brands_id = b.id
        WHERE b.owner_id = $1
        "#,
        uuid,
    )
    .fetch_all(pool)
    .await?;

    Ok(res)
}

pub async fn svc_get_parfume_by_id(
    pool: &PgPool,
    access: &AccesClaims,
    id: &Uuid,
) -> Result<ParfumeResponse, AppError> {
    let uuid = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_get_parfume_by_id: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    let res = match sqlx::query_as!(
        ParfumeResponse,
        r#"
        SELECT
            p.id,
            p.brands_id,
            b.name AS "brands_name!",
            p.name,
            p.concentration,
            p.description
        FROM parfume p
        JOIN brands b
            ON p.brands_id = b.id
        WHERE p.id = $1
        AND b.owner_id = $2
        "#,
        id,
        uuid
    )
    .fetch_optional(pool)
    .await?
    {
        Some(val) => val,
        None => {
            return Err(AppError::NotFound(
                None,
                Some("svc_get_parfume_by_id: parfume tidak ditemukan".to_string()),
            ));
        }
    };

    Ok(res)
}
