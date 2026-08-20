use sqlx::{PgPool, query_as};
use uuid::Uuid;
use validator::Validate;

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::{request::request_user::CreateBrands, response::response_user::Brand},
    error::error::AppError,
    handlers::brand,
};

pub async fn svc_create_brands(
    pool: &PgPool,
    req: &CreateBrands,
    access: &AccesClaims,
) -> Result<String, AppError> {
    let name = req.name_brands.trim();

    if name.is_empty() {
        return Err(AppError::BadRequest(None));
    }

    let validate = match req.validate() {
        Ok(_) => {}
        Err(_) => return Err(AppError::BadRequest(None)),
    };

    let owner_uuid = match Uuid::parse_str(&access.sub) {
        Ok(val) => val,
        Err(_) => return Err(AppError::InternalServerError(None)),
    };

    let result = sqlx::query!(
        r#"INSERT INTO brands (owner_id, name) VALUES ($1, $2)"#,
        owner_uuid,
        name
    )
    .execute(pool)
    .await?;

    Ok("Success".to_string())
}

pub async fn svc_get_all_brands(
    pool: &PgPool,
    access: &AccesClaims,
) -> Result<Vec<Brand>, AppError> {
    let uuid = match Uuid::parse_str(&access.sub) {
        Ok(val) => val,
        Err(e) => return Err(AppError::InternalServerError(Some(e.to_string()))),
    };

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
        WHERE b.owner_id = $1
        GROUP BY b.id, b.name
        ORDER BY b.name
    "#,
        uuid
    )
    .fetch_all(pool)
    .await?;

    Ok(brands)
}

/*
pub async fn svc_get_all_brands() -> Result<Vec<Brand>, AppError>{
    let brands = sqlx::query_as!(Brand, r#"SELECT FROM")
}
    */
