use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dto::{
        request::brand_req::{CreateBrands, UpdateBrands},
        response::brand_res::Brand,
    },
    error::error::AppError,
};

pub async fn svc_create_brands(
    pool: &PgPool,
    req: &CreateBrands,
    owner_id: Uuid,
) -> Result<Brand, AppError> {
    let name = req.name_brands.trim();

    if name.is_empty() {
        return Err(AppError::BadRequest(
            None,
            Some("svc_create_brands: nama brand kosong".to_string()),
        ));
    }

    if req.validate().is_err() {
        return Err(AppError::BadRequest(
            None,
            Some("svc_create_brands: validasi input gagal".to_string()),
        ));
    }

    let brand = sqlx::query_as!(
        Brand,
        r#"
        INSERT INTO brands (owner_id, name)
        VALUES ($1, $2)
        RETURNING
            id,
            name,
            0 AS "total_parfume!"
        "#,
        owner_id,
        name
    )
    .fetch_one(pool)
    .await?;

    Ok(brand)
}

pub async fn svc_get_all_brands(
    pool: &PgPool,
    owner_id: Uuid,
) -> Result<Vec<Brand>, AppError> {
    let brands = sqlx::query_as!(
        Brand,
        r#"
        SELECT
            b.id,
            b.name,
            COUNT(p.id)::INT AS total_parfume
        FROM brands b
        LEFT JOIN parfume p
            ON p.brands_id = b.id
           AND p.deleted_at IS NULL
        WHERE b.owner_id = $1
          AND b.deleted_at IS NULL
        GROUP BY b.id, b.name
        ORDER BY b.name
    "#,
        owner_id
    )
    .fetch_all(pool)
    .await?;

    Ok(brands)
}

pub async fn svc_update_brands(
    pool: &PgPool,
    req: &UpdateBrands,
    owner_id: Uuid,
    id: &Uuid,
) -> Result<Brand, AppError> {
    if id != &req.brands_id {
        return Err(AppError::Forbidden(
            None,
            Some("svc_update_brands: hanya Dev yang boleh".to_string()),
        ));
    };

    let brand = sqlx::query_as!(
        Brand,
        r#"
        UPDATE brands
        SET name = $1
        WHERE id = $2 AND owner_id = $3 AND deleted_at IS NULL
        RETURNING
            id,
            name,
            (SELECT COUNT(p.id)::INT FROM parfume p WHERE p.brands_id = brands.id AND p.deleted_at IS NULL) AS "total_parfume!"
        "#,
        req.name_brands,
        &req.brands_id,
        owner_id
    )
    .fetch_optional(pool)
    .await?;

    let brand = match brand {
        Some(val) => val,
        None => {
            return Err(AppError::NotFound(
                None,
                Some("svc_update_brands: brand tidak ditemukan".to_string()),
            ));
        }
    };

    Ok(brand)
}

pub async fn svc_get_brands_by_id(
    pool: &PgPool,
    owner_id: Uuid,
    id: &Uuid,
) -> Result<Option<Brand>, AppError> {
    let brand = sqlx::query_as!(
        Brand,
        r#"
        SELECT
            b.id,
            b.name,
            COUNT(p.id)::INT AS total_parfume
        FROM brands b
        LEFT JOIN parfume p
            ON p.brands_id = b.id
           AND p.deleted_at IS NULL
        WHERE b.owner_id = $1
          AND b.id = $2
          AND b.deleted_at IS NULL
        GROUP BY b.id, b.name
        ORDER BY b.name
    "#,
        owner_id,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(brand)
}

pub async fn svc_delete_brand(
    pool: &PgPool,
    owner_id: Uuid,
    id: &Uuid,
) -> Result<String, AppError> {
    let brand = sqlx::query!(
        r#"
        SELECT
            b.id AS "id!",
            EXISTS (
                SELECT 1
                FROM batch_parfume_bottle bf
                JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
                JOIN parfume p ON p.id = bp.parfume_id
                JOIN order_items oi ON oi.bottle_id = bf.id
                WHERE p.brands_id = b.id
            ) OR EXISTS (
                SELECT 1
                FROM batch_parfume_bottle bf
                JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
                JOIN parfume p ON p.id = bp.parfume_id
                JOIN stock_movements sm ON sm.bottle_id = bf.id
                WHERE p.brands_id = b.id
            ) OR EXISTS (
                SELECT 1
                FROM decant d
                JOIN parfume p ON p.id = d.parfume_id
                JOIN order_items oi ON oi.decant_id = d.id
                WHERE p.brands_id = b.id
            ) AS "has_history!"
        FROM brands b
        WHERE b.id = $1
          AND b.owner_id = $2
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
            Some("svc_delete_brand: brand tidak ditemukan".to_string()),
        )
    })?;

    if brand.has_history {
        let soft = sqlx::query!(
            r#"
            UPDATE brands
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
                Some("svc_delete_brand: brand sudah terhapus".to_string()),
            ));
        }

        return Ok("Berhasil dihapus".to_string());
    }

    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        DELETE FROM batch_parfume_bottle bf
        USING batch_parfume bp, parfume p
        WHERE bf.batch_parfume_id = bp.id
          AND bp.parfume_id = p.id
          AND p.brands_id = $1
        "#,
        id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM batch_parfume bp
        USING parfume p
        WHERE bp.parfume_id = p.id
          AND p.brands_id = $1
        "#,
        id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM decant d
        USING parfume p
        WHERE d.parfume_id = p.id
          AND p.brands_id = $1
        "#,
        id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!("DELETE FROM parfume WHERE brands_id = $1", id)
        .execute(&mut *tx)
        .await?;

    sqlx::query!("DELETE FROM brands WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok("Berhasil dihapus".to_string())
}
