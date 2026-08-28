use bigdecimal::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    dto::{
        request::batch_req::{CreateBatch, UpdateBatch},
        response::batch_res::BatchResponse,
    },
    error::error::AppError,
};

pub async fn svc_create_batch(
    pool: &PgPool,
    req: &CreateBatch,
    owner_id: Uuid,
    id: &Uuid,
) -> Result<BatchResponse, AppError> {
    if req.quantity_ml <= BigDecimal::from(0) {
        return Err(AppError::BadRequest(
            Some("quantity_ml harus lebih dari 0".to_string()),
            Some("svc_create_batch: quantity_ml <= 0".to_string()),
        ));
    }

    let batch = sqlx::query_as!(
        BatchResponse,
        r#"
        WITH inserted AS (
            INSERT INTO batch_parfume (
                parfume_id,
                quantity_ml,
                purchase_price
            )
            SELECT
                f.id,
                $3,
                $4
            FROM parfume f
            JOIN brands b
                ON f.brands_id = b.id
            WHERE f.id = $1
              AND b.owner_id = $2
              AND f.deleted_at IS NULL
              AND b.deleted_at IS NULL
            RETURNING
                id,
                parfume_id,
                quantity_ml,
                purchase_price
        )
        SELECT
            i.id,
            i.parfume_id,
            i.quantity_ml,
            i.purchase_price
        FROM inserted i
        "#,
        id,
        owner_id,
        req.quantity_ml,
        req.purchase_price
    )
    .fetch_optional(pool)
    .await?;

    let batch = match batch {
        Some(val) => val,
        None => {
            return Err(AppError::InternalServerError(
                Some("Batch not found".to_string()),
                Some("svc_create_batch: parfume_id tidak ditemukan / bukan milik user".to_string()),
            ));
        }
    };

    Ok(batch)
}

pub async fn svc_get_all_batch(
    pool: &PgPool,
    owner_id: Uuid,
    id: &Uuid,
) -> Result<Vec<BatchResponse>, AppError> {
    let res = sqlx::query_as!(
        BatchResponse,
        r#"
        SELECT
            bp.id,
            p.id AS parfume_id,
            bp.quantity_ml,
            bp.purchase_price
        FROM batch_parfume bp
        JOIN parfume p
            ON bp.parfume_id = p.id
        JOIN brands br
            ON p.brands_id = br.id
        WHERE bp.parfume_id = $1
        AND br.owner_id = $2
        AND bp.deleted_at IS NULL
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

pub async fn svc_update_batch(
    pool: &PgPool,
    req: &UpdateBatch,
    owner_id: Uuid,
    id: &Uuid,
) -> Result<BatchResponse, AppError> {

    let batch = sqlx::query!(
        r#"
        SELECT
            bf.quantity_ml,
            bf.purchase_price,
            COALESCE((
                SELECT MAX(bf2.remaining_ml)
                FROM batch_parfume_bottle bf2
                WHERE bf2.batch_parfume_id = bf.id
            ), 0) AS "max_remaining_ml!"
        FROM batch_parfume bf
        JOIN parfume f
            ON bf.parfume_id = f.id
        JOIN brands b
            ON f.brands_id = b.id
        WHERE b.owner_id = $1
          AND bf.id = $2
          AND bf.deleted_at IS NULL
          AND f.deleted_at IS NULL
          AND b.deleted_at IS NULL
    "#,
        owner_id,
        id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        AppError::NotFound(
            None,
            Some("svc_update_batch: batch tidak ditemukan".to_string()),
        )
    })?;

    let quantity_ml = match &req.quantity_ml {
        Some(val) => val,
        None => &batch.quantity_ml,
    };

    if quantity_ml < &batch.max_remaining_ml {
        return Err(AppError::BadRequest(
            Some("quantity_ml tidak boleh lebih kecil dari sisa ml botol terbesar".to_string()),
            Some("svc_update_batch: quantity_ml baru di bawah remaining_ml salah satu bottle".to_string()),
        ));
    }

    let purchase_price = match &req.purchase_price {
        Some(val) => val,
        None => &batch.purchase_price,
    };

    let batch = sqlx::query_as!(
        BatchResponse,
        r#"
        UPDATE batch_parfume
        SET quantity_ml = $1,
            purchase_price = $2
        WHERE id = $3
        RETURNING
            id,
            parfume_id,
            quantity_ml,
            purchase_price
        "#,
        quantity_ml,
        purchase_price,
        id
    )
    .fetch_optional(pool)
    .await?;

    let batch = match batch {
        Some(val) => val,
        None => {
            return Err(AppError::NotFound(
                None,
                Some("svc_update_batch: batch tidak ditemukan".to_string()),
            ));
        }
    };

    Ok(batch)
}

pub async fn svc_delete_batch(
    pool: &PgPool,
    owner_id: Uuid,
    id: &Uuid,
) -> Result<String, AppError> {
    let batch = sqlx::query!(
        r#"
        SELECT
            bp.id AS "id!",
            EXISTS (
                SELECT 1
                FROM batch_parfume_bottle bf
                WHERE bf.batch_parfume_id = bp.id
                  AND (
                      EXISTS (SELECT 1 FROM order_items oi WHERE oi.bottle_id = bf.id)
                   OR EXISTS (SELECT 1 FROM stock_movements sm WHERE sm.bottle_id = bf.id)
                  )
            ) AS "has_history!"
        FROM batch_parfume bp
        JOIN parfume f
            ON bp.parfume_id = f.id
        JOIN brands b
            ON f.brands_id = b.id
        WHERE bp.id = $1
          AND b.owner_id = $2
          AND bp.deleted_at IS NULL
          AND f.deleted_at IS NULL
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
            Some("svc_delete_batch: batch tidak ditemukan".to_string()),
        )
    })?;

    if batch.has_history {
        let soft = sqlx::query!(
            r#"
            UPDATE batch_parfume
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
                Some("svc_delete_batch: batch sudah terhapus".to_string()),
            ));
        }

        return Ok("Berhasil dihapus".to_string());
    }

    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        DELETE FROM batch_parfume_bottle
        WHERE batch_parfume_id = $1
        "#,
        id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM batch_parfume
        WHERE id = $1
        "#,
        id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok("Berhasil dihapus".to_string())
}
