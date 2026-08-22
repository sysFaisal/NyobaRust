use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::{
        request::botol_req::{BottleStatus, CreateBottle, UpdateBottle},
        response::botol_res::BotolResponse,
    },
    error::error::AppError,
};



pub async fn svc_create_bottle(pool: &PgPool, req: &CreateBottle, access: &AccesClaims, batch_id: &Uuid) -> Result<String, AppError> {

    if &req.batch_id != batch_id {
        return Err(AppError::Forbidden(
            None,
            Some("svc_create_bottle: hanya Dev yang boleh".to_string()),
        ));
    }

    let uuid = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => return Err(AppError::InternalServerError(None, Some("svc_create_bottle: gagal parse UUID dari claims".to_string()))),
    };
    
    let query = sqlx::query!(
    r#"
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
    "#,
    req.batch_id,
    req.remaining_ml,
    req.status as BottleStatus,
    uuid
    )
    .execute(pool)
    .await?;

    if query.rows_affected() == 0 {
        return Err(AppError::InternalServerError(
            None,
            Some("svc_create_bottle: bottle_id tidak ditemukan / bukan milik user".to_string()),
        ));
    };

    Ok("Created Bottle".to_string())
}

fn access_uuid(access: &AccesClaims, operation: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(&access.sub).map_err(|_| {
        AppError::InternalServerError(
            None,
            Some(format!("{operation}: gagal parse UUID dari claims")),
        )
    })
}

pub async fn svc_get_bottle(
    pool: &PgPool,
    access: &AccesClaims,
    id: &Uuid,
) -> Result<BotolResponse, AppError> {
    let owner_id = access_uuid(access, "svc_get_bottle")?;
    sqlx::query_as!(
        BotolResponse,
        r#"
        SELECT
            bf.id,
            bf.batch_parfume_id,
            bf.remaining_ml,
            COALESCE(bf.status::text, '') AS "status!"
        FROM batch_parfume_bottle bf
        JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
        JOIN parfume p ON p.id = bp.parfume_id
        JOIN brands b ON b.id = p.brands_id
        WHERE bf.id = $1 AND b.owner_id = $2
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
    access: &AccesClaims,
    id: &Uuid,
    req: &UpdateBottle,
) -> Result<String, AppError> {
    let owner_id = access_uuid(access, "svc_update_bottle")?;
    let status = match &req.status {
        Some(status) => Some(match status.to_lowercase().as_str() {
            "available" => BottleStatus::Available,
            "empty" => BottleStatus::Empty,
            "inactive" => BottleStatus::Inactive,
            _ => return Err(AppError::BadRequest(None, Some("svc_update_bottle: status tidak valid".to_string()))),
        }),
        None => None,
    };

    let result = sqlx::query!(
        r#"
        UPDATE batch_parfume_bottle bf
        SET remaining_ml = COALESCE($1, bf.remaining_ml),
            status = COALESCE($2, bf.status)
        FROM batch_parfume bp
        JOIN parfume p ON p.id = bp.parfume_id
        JOIN brands b ON b.id = p.brands_id
        WHERE bf.id = $3
          AND b.owner_id = $4
        "#,
        req.remaining_ml,
        status as Option<BottleStatus>,
        id,
        owner_id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(None, Some("svc_update_bottle: bottle tidak ditemukan".to_string())));
    }

    Ok("Berhasil".to_string())
}

pub async fn svc_delete_bottle(
    pool: &PgPool,
    access: &AccesClaims,
    id: &Uuid,
) -> Result<String, AppError> {
    let owner_id = access_uuid(access, "svc_delete_bottle")?;
    let result = sqlx::query!(
        r#"
        DELETE FROM batch_parfume_bottle bf
        USING batch_parfume bp, parfume p, brands b
        WHERE bf.id = $1
          AND bp.id = bf.batch_parfume_id
          AND p.id = bp.parfume_id
          AND b.id = p.brands_id
          AND b.owner_id = $2
        "#,
        id,
        owner_id
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
    access: &AccesClaims,
    id: &Uuid,
) -> Result<Vec<BotolResponse>, AppError> {
    let owner_id = access_uuid(access, "svc_get_all_bottle")?;
    let res = sqlx::query_as!(
        BotolResponse,
        r#"
        SELECT
            bf.id,
            bf.batch_parfume_id,
            bf.remaining_ml,
            COALESCE(bf.status::text, '') AS "status!"
        FROM batch_parfume_bottle bf
        JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
        JOIN parfume p ON p.id = bp.parfume_id
        JOIN brands b ON b.id = p.brands_id
        WHERE bf.batch_parfume_id = $1
        AND b.owner_id = $2
        "#,
        id,
        owner_id
    )
    .fetch_all(pool)
    .await?;

    Ok(res)
}