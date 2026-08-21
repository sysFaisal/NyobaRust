

use sqlx::{PgPool, query};
use uuid::Uuid;

use crate::{c_auth::refresh_token::AccesClaims, dto::request::request_user::Decant, error::error::AppError};


pub async fn svc_create_decant(pool: &PgPool, req: &Decant, access: &AccesClaims) -> Result<String, AppError> {

    let owner = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => return Err(AppError::InternalServerError(None)),
    };

    let query = sqlx::query!(
        r#"
        INSERT INTO decant (
            parfume_id,
            size_ml,
            sell_price,
            is_active
        )
        SELECT
            f.id,
            $3,
            $4,
            $5
        FROM parfume f
        JOIN brands b
            ON f.brands_id = b.id
        WHERE f.id = $1
          AND b.owner_id = $2
        "#,
        req.parfume_id,
        owner,
        req.size_ml,
        req.sell_price,
        req.is_active
    )
    .execute(pool)
    .await?;

    if query.rows_affected() == 0 {
        return Err(AppError::InternalServerError(Some(
            "Decant not found".to_string(),
        )));
    };

    Ok("Created Decant".to_string())
}