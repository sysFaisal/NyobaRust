use bigdecimal::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    dto::{
        request::botol_req::BottleStatus, request::order_req::CreateOrder,
        response::order_mod::{OrderResponse, OrderStatus},
    },
    error::error::AppError,
    service::bottle_svc::{MovementReason, MovementType},
};

pub async fn svc_create_order(
    pool: &PgPool,
    req: &CreateOrder,
    owner_id: Uuid,
) -> Result<String, AppError> {
    if req.quantity < 1 {
        return Err(AppError::BadRequest(
            None,
            Some("svc_create_order: quantity tidak valid".to_string()),
        ));
    }

    // Single round-trip: lookup decant + matching bottle.
    // Returns 0 rows when decant missing; 1 row when found (LEFT JOIN on bottle).
    struct Lookup {
        decant_id: Uuid,
        parfume_id: Uuid,
        size_ml: BigDecimal,
        sell_price: BigDecimal,
        is_active: bool,
        bottle_id: Option<Uuid>,
        remaining_ml: Option<BigDecimal>,
        bottle_status: Option<BottleStatus>,
    }

    let row = sqlx::query_as!(
        Lookup,
        r#"
        SELECT
            d.id AS decant_id,
            d.parfume_id AS parfume_id,
            d.size_ml AS "size_ml!",
            d.sell_price AS "sell_price!",
            d.is_active AS "is_active!",
            bf.id AS bottle_id,
            bf.remaining_ml AS "remaining_ml?",
            bf.status AS "bottle_status: BottleStatus"
        FROM decant d
        JOIN parfume f ON f.id = d.parfume_id
        JOIN brands br ON br.id = f.brands_id
        LEFT JOIN batch_parfume_bottle bf ON bf.id = $3
        LEFT JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
        LEFT JOIN parfume p ON p.id = bp.parfume_id
        LEFT JOIN brands b ON b.id = p.brands_id
        WHERE d.id = $1
          AND br.owner_id = $2
          AND d.deleted_at IS NULL
          AND f.deleted_at IS NULL
          AND br.deleted_at IS NULL
          AND (
              bf.id IS NULL
              OR (
                  b.owner_id = $2
                  AND bp.parfume_id = d.parfume_id
                  AND bf.deleted_at IS NULL
                  AND bp.deleted_at IS NULL
                  AND p.deleted_at IS NULL
                  AND b.deleted_at IS NULL
              )
          )
        "#,
        req.decant_id,
        owner_id,
        req.bottle_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        AppError::NotFound(
            None,
            Some("svc_create_order: decant tidak ditemukan".to_string()),
        )
    })?;

    if !row.is_active {
        return Err(AppError::BadRequest(
            None,
            Some("svc_create_order: decant tidak aktif".to_string()),
        ));
    }

    let (bottle_id, remaining_ml, bottle_status) = match (
        row.bottle_id,
        row.remaining_ml,
        row.bottle_status,
    ) {
        (Some(id), Some(ml), Some(st)) => (id, ml, st),
        _ => {
            return Err(AppError::NotFound(
                None,
                Some("svc_create_order: bottle tidak ditemukan / bukan milik user / bukan di parfume yang sama".to_string()),
            ));
        }
    };

    if bottle_status != BottleStatus::Available {
        return Err(AppError::BadRequest(
            None,
            Some("svc_create_order: bottle tidak tersedia".to_string()),
        ));
    }

    let consumption_bd = &row.size_ml * BigDecimal::from(req.quantity);

    if consumption_bd > remaining_ml {
        return Err(AppError::BadRequest(
            None,
            Some("svc_create_order: bottle tidak cukup".to_string()),
        ));
    }

    let total_price = &row.sell_price * BigDecimal::from(req.quantity);

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
        bottle_id,
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
        bottle_id,
        req.decant_id,
        total_price,
        req.quantity,
        row.sell_price,
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
        bottle_id,
        consumption_bd,
        MovementType::Out as MovementType,
        MovementReason::Sale as MovementReason,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok("berhasil".to_string())
}

pub async fn svc_update_order_status(
    pool: &PgPool,
    order_id: &Uuid,
    target_status: OrderStatus,
    owner_id: Uuid,
) -> Result<OrderResponse, AppError> {
    // Hanya Failed dan Refund yang boleh via edit; Success/Pending ditolak
    if target_status != OrderStatus::Failed && target_status != OrderStatus::Refund {
        return Err(AppError::BadRequest(
            Some("status hanya boleh failed atau refund".to_string()),
            Some("svc_update_order_status: target_status harus failed/refund".to_string()),
        ));
    }

    // Fast path tanpa lock: bila status sudah sama, kembalikan langsung
    // tanpa membuka transaksi / mengambil FOR UPDATE.
    let peek = sqlx::query!(
        r#"
        SELECT
            oi.id AS "id!",
            oi.status AS "status: OrderStatus",
            oi.bottle_id AS "bottle_id!",
            oi.decant_id AS "decant_id!",
            oi.quantity AS "quantity!",
            oi.total_price AS "total_price!",
            oi.price AS "price!",
            oi.created_at AS "created_at!",
            d.size_ml AS "size_ml!",
            p.name AS "parfume_name!",
            br.name AS "brands_name!"
        FROM order_items oi
        JOIN batch_parfume_bottle bf ON bf.id = oi.bottle_id
        JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
        JOIN parfume p ON p.id = bp.parfume_id
        JOIN brands br ON br.id = p.brands_id
        JOIN decant d ON d.id = oi.decant_id
        WHERE oi.id = $1
          AND br.owner_id = $2
        "#,
        order_id,
        owner_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        AppError::NotFound(
            None,
            Some("svc_update_order_status: order tidak ditemukan / bukan milik user".to_string()),
        )
    })?;

    if peek.status == target_status {
        return Ok(OrderResponse {
            id: peek.id,
            bottle_id: peek.bottle_id,
            decant_id: peek.decant_id,
            total_price: peek.total_price,
            quantity: peek.quantity,
            price: peek.price,
            status: peek.status,
            size_ml: peek.size_ml,
            parfume_name: peek.parfume_name,
            brands_name: peek.brands_name,
            created_at: peek.created_at,
        });
    }

    let mut tx = pool.begin().await?;

    // Lock order + bottle + batch untuk cegah race, verifikasi owner
    let row = sqlx::query!(
        r#"
        SELECT
            oi.id AS "id!",
            oi.status AS "status: OrderStatus",
            oi.bottle_id AS "bottle_id!",
            oi.decant_id AS "decant_id!",
            oi.quantity AS "quantity!",
            d.size_ml AS "size_ml!",
            (d.size_ml * oi.quantity) AS "consumption!",
            bf.remaining_ml AS "remaining_ml!",
            oi.total_price AS "total_price!",
            oi.price AS "price!",
            oi.created_at AS "created_at!",
            p.name AS "parfume_name!",
            br.name AS "brands_name!"
        FROM order_items oi
        JOIN batch_parfume_bottle bf ON bf.id = oi.bottle_id
        JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
        JOIN parfume p ON p.id = bp.parfume_id
        JOIN brands br ON br.id = p.brands_id
        JOIN decant d ON d.id = oi.decant_id
        WHERE oi.id = $1
          AND br.owner_id = $2
        FOR UPDATE OF oi, bf
        "#,
        order_id,
        owner_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| {
        AppError::NotFound(
            None,
            Some("svc_update_order_status: order tidak ditemukan / bukan milik user".to_string()),
        )
    })?;

    let current = row.status;

    // Hanya dari Success yang boleh ke Failed/Refund
    if current != OrderStatus::Success {
        tx.rollback().await?;
        return Err(AppError::BadRequest(
            Some(format!(
                "hanya order dengan status success yang bisa diubah ke {}",
                match target_status {
                    OrderStatus::Failed => "failed",
                    OrderStatus::Refund => "refund",
                    _ => "target",
                }
            )),
            Some(format!(
                "svc_update_order_status: transisi {:?} -> {:?} tidak diizinkan",
                current, target_status
            )),
        ));
    }

    let consumption_bd = BigDecimal::from(row.consumption);

    // Cek kapasitas sebelum update (hindari overflow remaining > quantity_ml)
    if &row.remaining_ml + &consumption_bd > row.size_ml * row.quantity {
        tx.rollback().await?;
        return Err(AppError::BadRequest(
            Some("refund melebihi kapasitas botol (quantity_ml batch)".to_string()),
            Some("svc_update_order_status: remaining_ml + consumption > quantity_ml".to_string()),
        ));
    }

    sqlx::query!(
        r#"
        UPDATE order_items
        SET status = $2
        WHERE id = $1
        "#,
        order_id,
        target_status as OrderStatus
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        UPDATE batch_parfume_bottle
        SET remaining_ml = remaining_ml + $2
        WHERE id = $1
        "#,
        row.bottle_id,
        consumption_bd
    )
    .execute(&mut *tx)
    .await?;

    // INSERT baru, bukan UPDATE — ledger append-only
    sqlx::query!(
        r#"
        INSERT INTO stock_movements (order_items_id, bottle_id, quantity, type, reason)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        order_id,
        row.bottle_id,
        consumption_bd,
        MovementType::In as MovementType,
        MovementReason::Refund as MovementReason,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(OrderResponse {
        id: row.id,
        bottle_id: row.bottle_id,
        decant_id: row.decant_id,
        total_price: row.total_price,
        quantity: row.quantity,
        price: row.price,
        status: target_status,
        size_ml: row.size_ml,
        parfume_name: row.parfume_name,
        brands_name: row.brands_name,
        created_at: row.created_at,
    })
}

pub const ORDER_PER_PAGE: i64 = 10;

pub async fn svc_get_all_order(
    pool: &PgPool,
    owner_id: Uuid,
    page: i64,
    per_page: i64,
) -> Result<(Vec<OrderResponse>, i64), AppError> {
    let offset = page.saturating_sub(1).saturating_mul(per_page);

    let total_items = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!"
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
        "#,
        owner_id
    )
    .fetch_one(pool)
    .await?;

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
        LIMIT $2 OFFSET $3
        "#,
        owner_id,
        per_page,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok((res, total_items))
}
