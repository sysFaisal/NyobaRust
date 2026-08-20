use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::{request::request_user::CreateBrands, response::response_user::Brand},
    error::error::AppError,
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
        Err(_) => return Err(AppError::InternalServerError(None)) 
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
/*
pub async fn svc_get_all_brands() -> Result<Vec<Brand>, AppError>{
    let brands = sqlx::query_as!(Brand, r#"SELECT FROM")
}
    */
