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
        AND p.deleted_at IS NULL
        AND b.deleted_at IS NULL
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
      AND b.deleted_at IS NULL
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
        AND p.deleted_at IS NULL
        AND b.deleted_at IS NULL
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
        AND p.deleted_at IS NULL
        AND b.deleted_at IS NULL
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

pub async fn svc_delete_parfume(
    pool: &PgPool,
    access: &AccesClaims,
    id: &Uuid,
) -> Result<String, AppError> {
    let uuid = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_delete_parfume: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    let parfume = sqlx::query!(
        r#"
        SELECT
            p.id AS "id!",
            EXISTS (
                SELECT 1
                FROM batch_parfume_bottle bf
                JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
                JOIN order_items oi ON oi.bottle_id = bf.id
                WHERE bp.parfume_id = p.id
            ) OR EXISTS (
                SELECT 1
                FROM batch_parfume_bottle bf
                JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
                JOIN stock_movements sm ON sm.bottle_id = bf.id
                WHERE bp.parfume_id = p.id
            ) OR EXISTS (
                SELECT 1
                FROM decant d
                JOIN order_items oi ON oi.decant_id = d.id
                WHERE d.parfume_id = p.id
            ) AS "has_history!"
        FROM parfume p
        JOIN brands b ON b.id = p.brands_id
        WHERE p.id = $1
          AND b.owner_id = $2
          AND p.deleted_at IS NULL
          AND b.deleted_at IS NULL
        "#,
        id,
        uuid
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        AppError::NotFound(
            None,
            Some("svc_delete_parfume: parfume tidak ditemukan".to_string()),
        )
    })?;

    if parfume.has_history {
        let soft = sqlx::query!(
            r#"
            UPDATE parfume
            SET deleted_at = now()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .execute(pool)
        .await?;

        if soft.rows_affected() == 0 {
            return Err(AppError::Conflict(
                None,
                Some("svc_delete_parfume: parfume sudah terhapus".to_string()),
            ));
        }

        return Ok("Berhasil dihapus".to_string());
    }

    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        DELETE FROM batch_parfume_bottle bf
        USING batch_parfume bp
        WHERE bf.batch_parfume_id = bp.id
          AND bp.parfume_id = $1
        "#,
        id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!("DELETE FROM batch_parfume WHERE parfume_id = $1", id)
        .execute(&mut *tx)
        .await?;

    sqlx::query!("DELETE FROM decant WHERE parfume_id = $1", id)
        .execute(&mut *tx)
        .await?;

    sqlx::query!("DELETE FROM parfume WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok("Berhasil dihapus".to_string())
}
