use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::{
        request::batch_req::{CreateBatch, UpdateBatch},
        response::batch_res::BatchResponse,
    },
    error::error::AppError,
};
/*
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
    "#
*/
pub async fn svc_create_batch(
    pool: &PgPool,
    req: &CreateBatch,
    access: &AccesClaims,
    id: &Uuid,
) -> Result<String, AppError> {
    let uuid = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_create_batch: gagal parse UUID dari claims".to_string()),
            ));
        }
    };
    let query = sqlx::query!(
        r#"
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
    "#,
        id,
        uuid,
        req.quantity_ml,
        req.purchase_price
    )
    .execute(pool)
    .await?;

    if query.rows_affected() == 0 {
        return Err(AppError::InternalServerError(
            Some("Batch not found".to_string()),
            Some("svc_create_batch: parfume_id tidak ditemukan / bukan milik user".to_string()),
        ));
    };

    Ok("Created Batch".to_string())
}

pub async fn svc_get_all_batch(
    pool: &PgPool,
    access: &AccesClaims,
    id: &Uuid,
) -> Result<Vec<BatchResponse>, AppError> {
    let uuid = match Uuid::parse_str(&access.sub) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_get_all_batch: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

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
        "#,
        id,
        uuid
    )
    .fetch_all(pool)
    .await?;

    Ok(res)
}

pub async fn svc_update_batch(
    pool: &PgPool,
    req: &UpdateBatch,
    access: &AccesClaims,
    id: &Uuid,
) -> Result<String, AppError> {

    let uuid = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_update_batch: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    let batch = sqlx::query!(
        r#"
        SELECT
            bf.quantity_ml,
            bf.purchase_price
        FROM batch_parfume bf
        JOIN parfume f
            ON bf.parfume_id = f.id
        JOIN brands b
            ON f.brands_id = b.id
        WHERE b.owner_id = $1
          AND bf.id = $2
    "#,
        uuid,
        id,
    )
    .fetch_one(pool)
    .await?;

    let quantity_ml = match &req.quantity_ml {
        Some(val) => val,
        None => &batch.quantity_ml,
    };

    let purchase_price = match &req.purchase_price {
        Some(val) => val,
        None => &batch.purchase_price,
    };

    let result = sqlx::query!(
        r#"
        UPDATE batch_parfume
        SET quantity_ml = $1,
            purchase_price = $2
        WHERE id = $3
        "#,
        quantity_ml,
        purchase_price,
        id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            None,
            Some("svc_update_batch: batch tidak ditemukan".to_string()),
        ));
    }

    Ok("Berhasil".to_string())
}
