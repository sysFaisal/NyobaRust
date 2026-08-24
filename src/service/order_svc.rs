use bigdecimal::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::{
        request::{botol_req::BottleStatus, order_req::CreateOrder},
        response::{
            botol_res::BotolResponse,
            decant_res::DecantResponse,
            order_mod::{OrderResponse, OrderStatus},
        },
    },
    error::error::AppError,
    service::bottle_svc::{MovementReason, MovementType},
};

pub async fn svc_create_order(
    pool: &PgPool,
    req: &CreateOrder,
    access: &AccesClaims,
) -> Result<String, AppError> {
    let uuid = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_create_order: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    if req.quantity < 1 {
        return Err(AppError::BadRequest(
            None,
            Some("svc_create_order: quantity tidak valid".to_string()),
        ));
    }

    let decant = match sqlx::query_as!(
        DecantResponse,
        r#"
        SELECT
            d.id,
            d.parfume_id,
            d.size_ml,
            d.sell_price,
            d.is_active
        FROM decant d
        JOIN parfume f
            ON d.parfume_id = f.id
        JOIN brands br
            ON f.brands_id = br.id
        WHERE d.id = $1
          AND br.owner_id = $2
          AND d.deleted_at IS NULL
          AND f.deleted_at IS NULL
          AND br.deleted_at IS NULL"#,
        req.decant_id,
        uuid
    )
    .fetch_optional(pool)
    .await?
    {
        Some(val) => val,
        None => {
            return Err(AppError::NotFound(
                None,
                Some("svc_create_order: decant tidak ditemukan".to_string()),
            ));
        }
    };

    if decant.is_active == false {
        return Err(AppError::BadRequest(
            None,
            Some("svc_create_order: decant tidak aktif".to_string()),
        ));
    };

    tracing::debug!(
        bottle_id = %req.bottle_id,
        owner_id = %uuid,
        parfume_id = %decant.parfume_id,
        "svc_create_order: mencari bottle"
    );

    let bottle = match sqlx::query_as!(
        BotolResponse,
        r#"
        SELECT
            bf.id,
            bf.batch_parfume_id,
            bf.remaining_ml,
            bf.status AS "status: BottleStatus"
        FROM batch_parfume_bottle bf
        JOIN batch_parfume bp
            ON bp.id = bf.batch_parfume_id
        JOIN parfume p
            ON p.id = bp.parfume_id
        JOIN brands b
            ON b.id = p.brands_id
        WHERE bf.id = $1
          AND b.owner_id = $2
          AND bp.parfume_id = $3
          AND bf.deleted_at IS NULL
          AND bp.deleted_at IS NULL
          AND p.deleted_at IS NULL
          AND b.deleted_at IS NULL
        "#,
        req.bottle_id,
        uuid,
        decant.parfume_id
    )
    .fetch_optional(pool)
    .await?
    {
        Some(val) => {
            tracing::debug!(
                bottle_id = %val.id,
                remaining_ml = %val.remaining_ml,
                status = ?val.status,
                "svc_create_order: bottle ditemukan"
            );
            val
        }
        None => {
            tracing::warn!(
                bottle_id = %req.bottle_id,
                owner_id = %uuid,
                parfume_id = %decant.parfume_id,
                "svc_create_order: bottle tidak ditemukan / sudah dihapus / bukan milik user"
            );
            return Err(AppError::NotFound(
                None,
                Some("svc_create_order: bottle tidak ditemukan".to_string()),
            ));
        }
    };

    if bottle.status != BottleStatus::Available {
        return Err(AppError::BadRequest(
            None,
            Some("svc_create_order: bottle tidak tersedia".to_string()),
        ));
    }

    let consumption_ml = decant.size_ml * req.quantity;
    let consumption_bd = BigDecimal::from(consumption_ml);

    if consumption_bd > bottle.remaining_ml {
        return Err(AppError::BadRequest(
            None,
            Some("svc_create_order: bottle tidak cukup".to_string()),
        ));
    }

    let total_price = &decant.sell_price * req.quantity;

    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        UPDATE batch_parfume_bottle bf
        SET remaining_ml = bf.remaining_ml - $2
        WHERE bf.id = $1
          AND bf.remaining_ml >= $2
          AND bf.status = 'available'
          AND bf.deleted_at IS NULL
        RETURNING bf.remaining_ml AS "remaining_ml!"
        "#,
        req.bottle_id,
        consumption_bd
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| {
        AppError::Conflict(
            Some("Stok botol berubah, stok tidak cukup".to_string()),
            Some("svc_create_order: gagal deduct remaining_ml (race / stok kurang)".to_string()),
        )
    })?;

    let inserted = sqlx::query!(
        r#"
        INSERT INTO order_items (bottle_id, decant_id, total_price, quantity, price, status)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id AS "id!"
        "#,
        req.bottle_id,
        req.decant_id,
        total_price,
        req.quantity,
        decant.sell_price,
        OrderStatus::Success as OrderStatus
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO stock_movements (order_items_id, bottle_id, quantity, type, reason)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        inserted.id,
        req.bottle_id,
        consumption_bd,
        MovementType::Out as MovementType,
        MovementReason::Sale as MovementReason,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok("berhasil".to_string())
}

pub async fn svc_get_all_order(
    pool: &PgPool,
    access: &AccesClaims,
) -> Result<Vec<OrderResponse>, AppError> {
    let uuid = match Uuid::parse_str(&access.sub) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_get_all_order: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    let res = sqlx::query_as!(
        OrderResponse,
        r#"
        SELECT
            oi.id,
            oi.bottle_id,
            oi.decant_id,
            oi.total_price,
            oi.quantity,
            oi.price,
            oi.status AS "status: OrderStatus",
            d.size_ml,
            p.name AS parfume_name,
            br.name AS brands_name,
            oi.created_at
        FROM order_items oi
        JOIN batch_parfume_bottle bf
            ON bf.id = oi.bottle_id
        JOIN batch_parfume bp
            ON bp.id = bf.batch_parfume_id
        JOIN parfume p
            ON p.id = bp.parfume_id
        JOIN brands br
            ON br.id = p.brands_id
        JOIN decant d
            ON d.id = oi.decant_id
        WHERE br.owner_id = $1
        ORDER BY oi.created_at DESC
        "#,
        uuid
    )
    .fetch_all(pool)
    .await?;

    Ok(res)
}
