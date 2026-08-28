use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    dto::{
        request::decant_req::{CreateDecant, UpdateDecant},
        response::decant_res::DecantResponse,
    },
    error::error::AppError,
};

pub async fn svc_create_decant(
    pool: &PgPool,
    req: &CreateDecant,
    owner_id: Uuid,
    id: &Uuid,
) -> Result<String, AppError> {
    let query = sqlx::query!(
        r#"
        INSERT INTO decant (
            parfume_id,
            size_ml,
            sell_price,
            is_active
        )
        SELECT
            f.id,
            $3,
            $4,
            $5
        FROM parfume f
        JOIN brands b
            ON f.brands_id = b.id
        WHERE f.id = $1
          AND b.owner_id = $2
          AND f.deleted_at IS NULL
          AND b.deleted_at IS NULL
        "#,
        id,
        owner_id,
        req.size_ml,
        req.sell_price,
        req.is_active
    )
    .execute(pool)
    .await?;

    if query.rows_affected() == 0 {
        return Err(AppError::InternalServerError(
            Some("Decant not found".to_string()),
            Some("svc_create_decant: parfume_id tidak ditemukan / bukan milik user".to_string()),
        ));
    };

    Ok("Created Decant".to_string())
}

pub async fn svc_get_all_decant(
    pool: &PgPool,
    owner_id: Uuid,
    id: &Uuid,
) -> Result<Vec<DecantResponse>, AppError> {
    let res = sqlx::query_as!(
        DecantResponse,
        r#"
        SELECT
            d.id,
            p.id AS parfume_id,
            d.size_ml,
            d.sell_price,
            d.is_active
        FROM decant d
        JOIN parfume p
            ON d.parfume_id = p.id
        JOIN brands br
            ON p.brands_id = br.id
        WHERE d.parfume_id = $1
        AND br.owner_id = $2
        AND d.deleted_at IS NULL
        AND p.deleted_at IS NULL
        AND br.deleted_at IS NULL
        "#,
        id,
        owner_id
    )
    .fetch_all(pool)
    .await?;

    Ok(res)
}

pub async fn svc_update_decant(
    pool: &PgPool,
    req: &UpdateDecant,
    owner_id: Uuid,
    id: &Uuid,
) -> Result<String, AppError> {
    let res_fallback = match sqlx::query_as!(
        CreateDecant,
        r#"
        SELECT
            d.size_ml,
            d.sell_price,
            d.is_active
        FROM decant d
        JOIN parfume p ON d.parfume_id = p.id
        JOIN brands b ON p.brands_id = b.id
        WHERE b.owner_id = $1
          AND d.id = $2
          AND d.deleted_at IS NULL
          AND p.deleted_at IS NULL
          AND b.deleted_at IS NULL
        "#,
        owner_id,
        id
    )
    .fetch_optional(pool)
    .await?
    {
        Some(val) => val,
        None => {
            return Err(AppError::NotFound(
                None,
                Some("svc_update_decant: decant tidak ditemukan".to_string()),
            ));
        }
    };

    let size_ml = match &req.size_ml {
        Some(val) => val,
        None => &res_fallback.size_ml,
    };

    let sell_price = match &req.sell_price {
        Some(val) => val,
        None => &res_fallback.sell_price,
    };

    let is_active = match &req.is_active {
        Some(val) => val,
        None => &res_fallback.is_active,
    };

    let query = sqlx::query!(
        r#"
    UPDATE decant d
    SET size_ml = $1,
        sell_price = $2,
        is_active = $3
    FROM parfume p
    JOIN brands b
        ON p.brands_id = b.id
    WHERE d.parfume_id = p.id
      AND b.owner_id = $4
      AND d.id = $5
      AND d.deleted_at IS NULL
      AND p.deleted_at IS NULL
      AND b.deleted_at IS NULL
    "#,
        size_ml,
        sell_price,
        is_active,
        owner_id,
        id
    )
    .execute(pool)
    .await?;

    if query.rows_affected() == 0 {
        return Err(AppError::NotFound(
            None,
            Some("svc_update_decant: decant tidak ditemukan".to_string()),
        ));
    }

    Ok("Berhasil".to_string())
}

pub async fn svc_delete_decant(
    pool: &PgPool,
    owner_id: Uuid,
    id: &Uuid,
) -> Result<String, AppError> {
    let decant = sqlx::query!(
        r#"
        SELECT
            d.id AS "id!",
            EXISTS (SELECT 1 FROM order_items oi WHERE oi.decant_id = d.id) AS "has_order!"
        FROM decant d
        JOIN parfume p
            ON p.id = d.parfume_id
        JOIN brands b
            ON b.id = p.brands_id
        WHERE d.id = $1
          AND b.owner_id = $2
          AND d.deleted_at IS NULL
          AND p.deleted_at IS NULL
          AND b.deleted_at IS NULL
        "#,
        id,
        owner_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        AppError::NotFound(
            None,
            Some("svc_delete_decant: decant tidak ditemukan".to_string()),
        )
    })?;

    if decant.has_order {
        let soft = sqlx::query!(
            r#"
            UPDATE decant
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
                Some("svc_delete_decant: decant sudah terhapus".to_string()),
            ));
        }

        return Ok("Berhasil dihapus".to_string());
    }

    let result = sqlx::query!(
        r#"
        DELETE FROM decant d
        WHERE d.id = $1
        "#,
        id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            None,
            Some("svc_delete_decant: decant tidak ditemukan".to_string()),
        ));
    }

    Ok("Berhasil dihapus".to_string())
}
