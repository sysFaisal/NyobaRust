use sqlx::{PgPool, query};
use uuid::Uuid;

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::{
        request::decant_req::CreateDecant, request::decant_req::UpdateDecant,
        response::decant_res::DecantResponse,
    },
    error::error::AppError,
};

pub async fn svc_create_decant(
    pool: &PgPool,
    req: &CreateDecant,
    access: &AccesClaims,
    id: &Uuid,
) -> Result<String, AppError> {
    let owner = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_create_decant: gagal parse UUID dari claims".to_string()),
            ));
        }
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
        id,
        owner,
        req.size_ml,
        req.sell_price,
        req.is_active
    )
    .execute(pool)
    .await?;

    if query.rows_affected() == 0 {
        return Err(AppError::InternalServerError(
            Some("Decant not found".to_string()),
            Some("svc_create_decant: parfume_id tidak ditemukan / bukan milik user".to_string()),
        ));
    };

    Ok("Created Decant".to_string())
}

pub async fn svc_get_all_decant(
    pool: &PgPool,
    access: &AccesClaims,
    id: &Uuid,
) -> Result<Vec<DecantResponse>, AppError> {
    let uuid = match Uuid::parse_str(&access.sub) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_get_all_decant: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    let res = sqlx::query_as!(
        DecantResponse,
        r#"
        SELECT
            d.id,
            p.id AS parfume_id,
            d.size_ml,
            d.sell_price,
            d.is_active
        FROM decant d
        JOIN parfume p
            ON d.parfume_id = p.id
        JOIN brands br
            ON p.brands_id = br.id
        WHERE d.parfume_id = $1
        AND br.owner_id = $2
        "#,
        id,
        uuid
    )
    .fetch_all(pool)
    .await?;

    Ok(res)
}

pub async fn svc_update_decant(
    pool: &PgPool,
    req: &UpdateDecant,
    access: &AccesClaims,
    id: &Uuid,
) -> Result<String, AppError> {
    let owner = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_update_decant: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    let res_fallback = match sqlx::query_as!(
        CreateDecant,
        r#"
        SELECT
            d.size_ml,
            d.sell_price,
            d.is_active
        FROM decant d
        JOIN parfume p ON d.parfume_id = p.id
        JOIN brands b ON p.brands_id = b.id
        WHERE b.owner_id = $1
          AND d.id = $2
        "#,
        owner,
        id
    )
    .fetch_optional(pool)
    .await?
    {
        Some(val) => val,
        None => {
            return Err(AppError::NotFound(
                None,
                Some("svc_update_decant: decant tidak ditemukan".to_string()),
            ));
        }
    };

    let size_ml = match &req.size_ml {
        Some(val) => val,
        None => &res_fallback.size_ml,
    };

    let sell_price = match &req.sell_price {
        Some(val) => val,
        None => &res_fallback.sell_price,
    };

    let is_active = match &req.is_active {
        Some(val) => val,
        None => &res_fallback.is_active,
    };

    let query = sqlx::query!(
        r#"
    UPDATE decant d
    SET size_ml = $1,
        sell_price = $2,
        is_active = $3
    FROM parfume p
    JOIN brands b
        ON p.brands_id = b.id
    WHERE d.parfume_id = p.id
      AND b.owner_id = $4
      AND d.id = $5
    "#,
        size_ml,
        sell_price,
        is_active,
        owner,
        id
    )
    .execute(pool)
    .await?;

    if query.rows_affected() == 0 {
        return Err(AppError::NotFound(
            None,
            Some("svc_update_decant: decant tidak ditemukan".to_string()),
        ));
    }

    Ok("Berhasil".to_string())
}
