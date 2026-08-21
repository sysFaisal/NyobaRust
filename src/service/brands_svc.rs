use sqlx::{PgPool, query_as};
use uuid::Uuid;
use validator::Validate;

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::{
        request::brand_req::{CreateBrands, UpdateBrands},
        response::brand_res::Brand,
    },
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
        return Err(AppError::BadRequest(
            None,
            Some("svc_create_brands: nama brand kosong".to_string()),
        ));
    }

    let validate = match req.validate() {
        Ok(_) => {}
        Err(_) => {
            return Err(AppError::BadRequest(
                None,
                Some("svc_create_brands: validasi input gagal".to_string()),
            ));
        }
    };

    let owner_uuid = match Uuid::parse_str(&access.sub) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_create_brands: gagal parse UUID dari claims".to_string()),
            ));
        }
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
        Err(e) => return Err(AppError::InternalServerError(None, Some(e.to_string()))),
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

pub async fn svc_update_brands(
    pool: &PgPool,
    req: &UpdateBrands,
    access: &AccesClaims,
    id: &Uuid,
) -> Result<String, AppError> {
    if id != &req.brands_id {
        return Err(AppError::Forbidden(
            None,
            Some("svc_update_brands: hanya Dev yang boleh".to_string()),
        ));
    };

    let uuid = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_update_brands: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    let result = sqlx::query!(
        r#"
        UPDATE brands
        SET name = $1
        WHERE id = $2 AND owner_id = $3
        "#,
        req.name_brands,
        &req.brands_id,
        uuid
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            None,
            Some("svc_update_brands: brand tidak ditemukan".to_string()),
        ));
    }

    Ok("Berhasil".to_string())
}

pub async fn svc_get_brands_by_id(
    pool: &PgPool,
    access: &AccesClaims,
    id: &Uuid,
) -> Result<Option<Brand>, AppError> {
    let uuid = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_get_brands_by_id: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

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
        WHERE b.owner_id = $1
          AND b.id = $2
        GROUP BY b.id, b.name
        ORDER BY b.name
    "#,
        uuid,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(brand)
}
/*
pub async fn svc_get_all_brands() -> Result<Vec<Brand>, AppError>{
    let brands = sqlx::query_as!(Brand, r#"SELECT FROM")
}
    */
