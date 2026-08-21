use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    c_auth::refresh_token::AccesClaims, dto::request::request_user::Batch, error::error::AppError,
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
    req: &Batch,
    access: &AccesClaims,
) -> Result<String, AppError> {
    let uuid = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => return Err(AppError::InternalServerError(None)),
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
        req.parfume_id,
        uuid,
        req.quantity_ml,
        req.purchase_price
    )
    .execute(pool)
    .await?;

    if query.rows_affected() == 0 {
        return Err(AppError::InternalServerError(Some(
            "Batch not found".to_string(),
        )));
    };

    Ok("Created Batch".to_string())
}
