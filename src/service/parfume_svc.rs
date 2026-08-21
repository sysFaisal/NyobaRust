use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::request::parfume_req::CreateParfume,
    error::error::AppError,
};

pub fn validate_string(value: &str, trimmed: bool, min_length: usize) -> bool {
    let value = if trimmed { value.trim() } else { value };

    value.len() >= min_length
}

pub async fn svc_create_parfume(
    pool: &PgPool,
    req: &CreateParfume,
    access: &AccesClaims,
) -> Result<String, AppError> {
    if !validate_string(&req.name, true, 3) {
        return Err(AppError::BadRequest(None, Some("svc_create_parfume: nama parfume kurang dari 3 karakter".to_string())));
    }

    let concentration = match &req.concrentration {
        Some(val) => {
            if !validate_string(&val, true, 3) {
                return Err(AppError::BadRequest(None, Some("svc_create_parfume: concentration kurang dari 3 karakter".to_string())));
            }

            Some(val.trim().to_string())
        }

        None => None,
    };

    let desc = match &req.description {
        Some(val) => {
            if !validate_string(&val, true, 3) {
                return Err(AppError::BadRequest(None, Some("svc_create_parfume: description kurang dari 3 karakter".to_string())));
            }

            Some(val.trim().to_string())
        }

        None => None,
    };

    let uuid = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => return Err(AppError::InternalServerError(None, Some("svc_create_parfume: gagal parse UUID dari claims".to_string()))),
    };
    let result = sqlx::query!(
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
    "#,
        req.brands_id,
        uuid,
        req.name.trim(),
        concentration,
        desc,
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Forbidden(None, Some("svc_create_parfume: brands_id bukan milik user ini".to_string())));
    }

    Ok("Parfume created successfully".to_string())
}
