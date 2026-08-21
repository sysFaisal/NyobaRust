use sqlx::PgPool;
use uuid::Uuid;

use crate::{c_auth::refresh_token::AccesClaims, dto::request::botol_req::{BottleStatus, CreateBottle}, error::error::AppError};



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