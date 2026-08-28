use bigdecimal::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    dto::{
        request::botol_req::{BottleStatus, CreateBottle, UpdateBottle},
        response::botol_res::BotolResponse,
    },
    error::error::AppError,
};

#[derive(Debug, Clone, Copy, PartialEq, sqlx::Type)]
#[sqlx(type_name = "type_stock_movement", rename_all = "lowercase")]
pub enum MovementType {
    In,
    Out,
}

#[derive(Debug, Clone, Copy, PartialEq, sqlx::Type)]
#[sqlx(type_name = "reason_stock_movement", rename_all = "lowercase")]
pub enum MovementReason {
    Initial,
    Sale,
    Refund,
    Adjustment,
}



pub async fn svc_create_bottle(pool: &PgPool, req: &CreateBottle, owner_id: Uuid, batch_id: &Uuid) -> Result<BotolResponse, AppError> {

    if &req.batch_id != batch_id {
        return Err(AppError::Forbidden(
            None,
            Some("svc_create_bottle: hanya Dev yang boleh".to_string()),
        ));
    }

    if req.remaining_ml <= BigDecimal::from(0) {
        return Err(AppError::BadRequest(
            Some("remaining_ml harus lebih dari 0".to_string()),
            Some("svc_create_bottle: remaining_ml <= 0".to_string()),
        ));
    }

    let cap = sqlx::query!(
        r#"
        SELECT
            bp.quantity_ml
        FROM batch_parfume bp
        JOIN parfume f
            ON f.id = bp.parfume_id
        JOIN brands b
            ON b.id = f.brands_id
        WHERE b.owner_id = $1
          AND bp.id = $2
          AND bp.deleted_at IS NULL
          AND f.deleted_at IS NULL
          AND b.deleted_at IS NULL
        "#,
        owner_id,
        batch_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        AppError::NotFound(
            None,
            Some("svc_create_bottle: batch tidak ditemukan / bukan milik user".to_string()),
        )
    })?;

    if req.remaining_ml > cap.quantity_ml {
        return Err(AppError::BadRequest(
            Some("Sisa ml botol melebihi kapasitas batch".to_string()),
            Some("svc_create_bottle: remaining_ml melebihi quantity_ml batch".to_string()),
        ));
    }

    let mut tx = pool.begin().await?;

    let bottle = sqlx::query_as!(
        BotolResponse,
        r#"
        WITH inserted AS (
            INSERT INTO batch_parfume_bottle (
                batch_parfume_id,
                remaining_ml,
                status
            )
            SELECT
                bf.id,
                $2,
                $3
            FROM batch_parfume bf JOIN parfume f
                ON bf.parfume_id = f.id
            JOIN brands b
                ON f.brands_id = b.id
            WHERE b.owner_id = $4
              AND bf.id = $1
            RETURNING
                id,
                batch_parfume_id,
                remaining_ml,
                status
        )
        SELECT
            i.id,
            i.batch_parfume_id,
            i.remaining_ml,
            i.status AS "status: BottleStatus"
        FROM inserted i
        "#,
        req.batch_id,
        req.remaining_ml,
        req.status as BottleStatus,
        owner_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| {
        AppError::InternalServerError(
            None,
            Some("svc_create_bottle: bottle_id tidak ditemukan / bukan milik user".to_string()),
        )
    })?;

    let order_item_id: Option<Uuid> = None;

    sqlx::query!(
        r#"
        INSERT INTO stock_movements (
            order_items_id,
            bottle_id,
            quantity,
            type,
            reason
        )
        VALUES ($1, $2, $3, $4, $5)
        "#,
        order_item_id,
        bottle.id,
        req.remaining_ml,
        MovementType::In as MovementType,
        MovementReason::Initial as MovementReason,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(bottle)
}

pub async fn svc_get_bottle(
    pool: &PgPool,
    owner_id: Uuid,
    id: &Uuid,
) -> Result<BotolResponse, AppError> {
    sqlx::query_as!(
        BotolResponse,
        r#"
        SELECT
            bf.id,
            bf.batch_parfume_id,
            bf.remaining_ml,
            bf.status AS "status: BottleStatus"
        FROM batch_parfume_bottle bf
        JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
        JOIN parfume p ON p.id = bp.parfume_id
        JOIN brands b ON b.id = p.brands_id
        WHERE bf.id = $1 AND b.owner_id = $2
          AND bf.deleted_at IS NULL
          AND bp.deleted_at IS NULL
          AND p.deleted_at IS NULL
          AND b.deleted_at IS NULL
        "#,
        id,
        owner_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound(
        None,
        Some("svc_get_bottle: bottle tidak ditemukan".to_string()),
    ))
}

pub async fn svc_update_bottle(
    pool: &PgPool,
    owner_id: Uuid,
    id: &Uuid,
    req: &UpdateBottle,
) -> Result<BotolResponse, AppError> {
    let Some(new_ml) = &req.remaining_ml else {
        let bottle = sqlx::query_as!(
            BotolResponse,
            r#"
            UPDATE batch_parfume_bottle bf
            SET status = COALESCE($1, bf.status)
            FROM batch_parfume bp
            JOIN parfume p ON p.id = bp.parfume_id
            JOIN brands b ON b.id = p.brands_id
            WHERE bf.id = $2
              AND b.owner_id = $3
              AND bf.deleted_at IS NULL
              AND bp.deleted_at IS NULL
              AND p.deleted_at IS NULL
              AND b.deleted_at IS NULL
            RETURNING
                bf.id,
                bf.batch_parfume_id,
                bf.remaining_ml,
                bf.status AS "status: BottleStatus"
            "#,
            req.status as Option<BottleStatus>,
            id,
            owner_id
        )
        .fetch_optional(pool)
        .await?;

        let bottle = match bottle {
            Some(val) => val,
            None => {
                return Err(AppError::NotFound(None, Some("svc_update_bottle: bottle tidak ditemukan".to_string())));
            }
        };

        return Ok(bottle);
    };

    if *new_ml <= BigDecimal::from(0) {
        return Err(AppError::BadRequest(
            Some("remaining_ml harus lebih dari 0".to_string()),
            Some("svc_update_bottle: remaining_ml <= 0".to_string()),
        ));
    }

    let mut tx = pool.begin().await?;

    let cap = sqlx::query!(
        r#"
        SELECT
            bp.quantity_ml,
            target.remaining_ml AS "remaining_ml!"
        FROM batch_parfume_bottle target
        JOIN batch_parfume bp
            ON bp.id = target.batch_parfume_id
        JOIN parfume f
            ON f.id = bp.parfume_id
        JOIN brands b
            ON b.id = f.brands_id
        WHERE target.id = $1
          AND b.owner_id = $2
          AND target.deleted_at IS NULL
          AND bp.deleted_at IS NULL
          AND f.deleted_at IS NULL
          AND b.deleted_at IS NULL
        "#,
        id,
        owner_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| {
        AppError::NotFound(
            None,
            Some("svc_update_bottle: bottle tidak ditemukan".to_string()),
        )
    })?;

    if *new_ml > cap.quantity_ml {
        return Err(AppError::BadRequest(
            Some("Sisa ml botol melebihi kapasitas batch".to_string()),
            Some("svc_update_bottle: remaining_ml melebihi quantity_ml batch".to_string()),
        ));
    }

    let bottle = sqlx::query_as!(
        BotolResponse,
        r#"
        UPDATE batch_parfume_bottle bf
        SET remaining_ml = COALESCE($1, bf.remaining_ml),
            status = COALESCE($2, bf.status)
        WHERE bf.id = $3
        RETURNING
            bf.id,
            bf.batch_parfume_id,
            bf.remaining_ml,
            bf.status AS "status: BottleStatus"
        "#,
        req.remaining_ml,
        req.status as Option<BottleStatus>,
        id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let bottle = match bottle {
        Some(val) => val,
        None => {
            return Err(AppError::NotFound(None, Some("svc_update_bottle: bottle tidak ditemukan".to_string())));
        }
    };

    let delta = new_ml - &cap.remaining_ml;

    if delta != BigDecimal::from(0) {
        let zero = BigDecimal::from(0);
        let (movement_type, magnitude) = if delta > zero {
            (MovementType::In, delta.clone())
        } else {
            (MovementType::Out, zero - delta)
        };

        let order_item_id: Option<Uuid> = None;

        sqlx::query!(
            r#"
            INSERT INTO stock_movements (
                order_items_id,
                bottle_id,
                quantity,
                type,
                reason
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
            order_item_id,
            id,
            magnitude,
            movement_type as MovementType,
            MovementReason::Adjustment as MovementReason,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(bottle)
}

pub async fn svc_delete_bottle(
    pool: &PgPool,
    owner_id: Uuid,
    id: &Uuid,
) -> Result<String, AppError> {
    let bottle = sqlx::query!(
        r#"
        SELECT
            EXISTS (SELECT 1 FROM order_items oi WHERE oi.bottle_id = bf.id) AS "has_order!",
            EXISTS (SELECT 1 FROM stock_movements sm WHERE sm.bottle_id = bf.id) AS "has_movement!"
        FROM batch_parfume_bottle bf
        JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
        JOIN parfume p ON p.id = bp.parfume_id
        JOIN brands b ON b.id = p.brands_id
        WHERE bf.id = $1
          AND b.owner_id = $2
          AND bf.deleted_at IS NULL
          AND bp.deleted_at IS NULL
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
            Some("svc_delete_bottle: bottle tidak ditemukan".to_string()),
        )
    })?;

    if bottle.has_order || bottle.has_movement {
        let soft = sqlx::query!(
            r#"
            UPDATE batch_parfume_bottle
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
                Some("svc_delete_bottle: bottle sudah terhapus".to_string()),
            ));
        }

        return Ok("Berhasil dihapus".to_string());
    }

    let result = sqlx::query!(
        r#"
        DELETE FROM batch_parfume_bottle bf
        WHERE bf.id = $1
        "#,
        id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(None, Some("svc_delete_bottle: bottle tidak ditemukan".to_string())));
    }

    Ok("Berhasil dihapus".to_string())
}

pub async fn svc_get_all_bottle(
    pool: &PgPool,
    owner_id: Uuid,
    id: &Uuid,
) -> Result<Vec<BotolResponse>, AppError> {
    let res = sqlx::query_as!(
        BotolResponse,
        r#"
        SELECT
            bf.id,
            bf.batch_parfume_id,
            bf.remaining_ml,
            bf.status AS "status: BottleStatus"
        FROM batch_parfume_bottle bf
        JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
        JOIN parfume p ON p.id = bp.parfume_id
        JOIN brands b ON b.id = p.brands_id
        WHERE bf.batch_parfume_id = $1
        AND b.owner_id = $2
        AND bf.deleted_at IS NULL
        AND bp.deleted_at IS NULL
        AND p.deleted_at IS NULL
        AND b.deleted_at IS NULL
        "#,
        id,
        owner_id
    )
    .fetch_all(pool)
    .await?;

    Ok(res)
}
