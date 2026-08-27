use chrono::{Datelike, Duration as ChronoDuration, Months, NaiveDate, NaiveDateTime, Utc};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::{
        request::{
            parfume_req::{CreateParfume, RankingQuery},
            revenue_req::RevenueHistoryQuery,
        },
        response::{
            parfume_res::{ParfumeRankingPoint, ParfumeResponse},
            revenue_res::{HistoryFrame, RevenueHistoryPoint},
        },
    },
    error::error::AppError,
};

pub fn validate_string(value: &str, trimmed: bool, min_length: usize) -> bool {
    let value = if trimmed { value.trim() } else { value };

    value.len() >= min_length
}

pub async fn svc_get_all_parfume(
    pool: &PgPool,
    access: &AccesClaims,
    id: &Uuid,
) -> Result<Vec<ParfumeResponse>, AppError> {
    let uuid = match Uuid::parse_str(&access.sub) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_get_all_parfume: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    let res = sqlx::query_as!(
        ParfumeResponse,
        r#"
        SELECT
            p.id,
            p.brands_id,
            b.name AS "brands_name!",
            p.name,
            p.concentration,
            p.description
        FROM parfume p
        JOIN brands b
            ON p.brands_id = b.id
        WHERE p.brands_id = $1
        AND b.owner_id = $2
        AND p.deleted_at IS NULL
        AND b.deleted_at IS NULL
        "#,
        id,
        uuid
    )
    .fetch_all(pool)
    .await?;

    Ok(res)
}

pub async fn svc_create_parfume(
    pool: &PgPool,
    req: &CreateParfume,
    access: &AccesClaims,
) -> Result<ParfumeResponse, AppError> {
    if !validate_string(&req.name, true, 3) {
        return Err(AppError::BadRequest(
            None,
            Some("svc_create_parfume: nama parfume kurang dari 3 karakter".to_string()),
        ));
    }

    let concentration = match &req.concrentration {
        Some(val) => {
            if !validate_string(&val, true, 3) {
                return Err(AppError::BadRequest(
                    None,
                    Some("svc_create_parfume: concentration kurang dari 3 karakter".to_string()),
                ));
            }

            Some(val.trim().to_string())
        }

        None => None,
    };

    let desc = match &req.description {
        Some(val) => {
            if !validate_string(&val, true, 3) {
                return Err(AppError::BadRequest(
                    None,
                    Some("svc_create_parfume: description kurang dari 3 karakter".to_string()),
                ));
            }

            Some(val.trim().to_string())
        }

        None => None,
    };

    let uuid = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_create_parfume: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    let res = sqlx::query_as!(
        ParfumeResponse,
        r#"
        WITH inserted AS (
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
              AND b.deleted_at IS NULL
            RETURNING
                id,
                brands_id,
                name,
                concentration,
                description
        )
        SELECT
            i.id,
            i.brands_id,
            b.name AS "brands_name!",
            i.name,
            i.concentration,
            i.description
        FROM inserted i
        JOIN brands b ON b.id = i.brands_id
        "#,
        req.brands_id,
        uuid,
        req.name.trim(),
        concentration,
        desc,
    )
    .fetch_optional(pool)
    .await?;

    let parfume = match res {
        Some(val) => val,
        None => {
            return Err(AppError::Forbidden(
                None,
                Some("svc_create_parfume: brands_id bukan milik user ini".to_string()),
            ));
        }
    };

    Ok(parfume)
}

pub async fn svc_update_parfume(
    pool: &PgPool,
    req: &crate::dto::request::parfume_req::UpdateParfume,
    access: &AccesClaims,
    id: &Uuid,
) -> Result<ParfumeResponse, AppError> {
    let uuid = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_update_parfume: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    let concentration = match &req.concrentration {
        Some(Some(val)) => {
            if !validate_string(&val, true, 3) {
                return Err(AppError::BadRequest(
                    None,
                    Some("svc_update_parfume: concentration kurang dari 3 karakter".to_string()),
                ));
            }
            Some(val.trim().to_string())
        }
        Some(None) => None,
        None => None,
    };

    let desc = match &req.description {
        Some(Some(val)) => {
            if !validate_string(&val, true, 3) {
                return Err(AppError::BadRequest(
                    None,
                    Some("svc_update_parfume: description kurang dari 3 karakter".to_string()),
                ));
            }
            Some(val.trim().to_string())
        }
        Some(None) => None,
        None => None,
    };

    let parfume = sqlx::query_as!(
        ParfumeResponse,
        r#"
        UPDATE parfume p
        SET name = COALESCE($3, p.name),
            concentration = COALESCE($4, p.concentration),
            description = COALESCE($5, p.description)
        FROM brands b
        WHERE p.id = $1
          AND b.id = p.brands_id
          AND b.owner_id = $2
          AND p.deleted_at IS NULL
          AND b.deleted_at IS NULL
        RETURNING
            p.id,
            p.brands_id,
            b.name AS "brands_name!",
            p.name,
            p.concentration,
            p.description
        "#,
        id,
        uuid,
        req.name.as_ref().map(|s| s.trim()),
        concentration,
        desc,
    )
    .fetch_optional(pool)
    .await?;

    let parfume = match parfume {
        Some(val) => val,
        None => {
            return Err(AppError::NotFound(
                None,
                Some("svc_update_parfume: parfume tidak ditemukan".to_string()),
            ));
        }
    };

    Ok(parfume)
}

pub async fn svc_get_all_parfume_uni(
    pool: &PgPool,
    access: &AccesClaims,
) -> Result<Vec<ParfumeResponse>, AppError> {
    let uuid = match Uuid::parse_str(&access.sub) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_get_all_parfume_uni: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    let res = sqlx::query_as!(
        ParfumeResponse,
        r#"
        SELECT
            p.id,
            p.brands_id,
            b.name AS "brands_name!",
            p.name,
            p.concentration,
            p.description
        FROM parfume p
        JOIN brands b
            ON p.brands_id = b.id
        WHERE b.owner_id = $1
        AND p.deleted_at IS NULL
        AND b.deleted_at IS NULL
        "#,
        uuid,
    )
    .fetch_all(pool)
    .await?;

    Ok(res)
}

pub async fn svc_get_parfume_by_id(
    pool: &PgPool,
    access: &AccesClaims,
    id: &Uuid,
) -> Result<ParfumeResponse, AppError> {
    let uuid = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_get_parfume_by_id: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    let res = match sqlx::query_as!(
        ParfumeResponse,
        r#"
        SELECT
            p.id,
            p.brands_id,
            b.name AS "brands_name!",
            p.name,
            p.concentration,
            p.description
        FROM parfume p
        JOIN brands b
            ON p.brands_id = b.id
        WHERE p.id = $1
        AND b.owner_id = $2
        AND p.deleted_at IS NULL
        AND b.deleted_at IS NULL
        "#,
        id,
        uuid
    )
    .fetch_optional(pool)
    .await?
    {
        Some(val) => val,
        None => {
            return Err(AppError::NotFound(
                None,
                Some("svc_get_parfume_by_id: parfume tidak ditemukan".to_string()),
            ));
        }
    };

    Ok(res)
}

pub async fn svc_delete_parfume(
    pool: &PgPool,
    access: &AccesClaims,
    id: &Uuid,
) -> Result<String, AppError> {
    let uuid = match Uuid::parse_str(access.sub.as_str()) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_delete_parfume: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    let parfume = sqlx::query!(
        r#"
        SELECT
            p.id AS "id!",
            EXISTS (
                SELECT 1
                FROM batch_parfume_bottle bf
                JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
                JOIN order_items oi ON oi.bottle_id = bf.id
                WHERE bp.parfume_id = p.id
            ) OR EXISTS (
                SELECT 1
                FROM batch_parfume_bottle bf
                JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
                JOIN stock_movements sm ON sm.bottle_id = bf.id
                WHERE bp.parfume_id = p.id
            ) OR EXISTS (
                SELECT 1
                FROM decant d
                JOIN order_items oi ON oi.decant_id = d.id
                WHERE d.parfume_id = p.id
            ) AS "has_history!"
        FROM parfume p
        JOIN brands b ON b.id = p.brands_id
        WHERE p.id = $1
          AND b.owner_id = $2
          AND p.deleted_at IS NULL
          AND b.deleted_at IS NULL
        "#,
        id,
        uuid
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        AppError::NotFound(
            None,
            Some("svc_delete_parfume: parfume tidak ditemukan".to_string()),
        )
    })?;

    if parfume.has_history {
        let soft = sqlx::query!(
            r#"
            UPDATE parfume
            SET deleted_at = now()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .execute(pool)
        .await?;

        if soft.rows_affected() == 0 {
            return Err(AppError::Conflict(
                None,
                Some("svc_delete_parfume: parfume sudah terhapus".to_string()),
            ));
        }

        return Ok("Berhasil dihapus".to_string());
    }

    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        DELETE FROM batch_parfume_bottle bf
        USING batch_parfume bp
        WHERE bf.batch_parfume_id = bp.id
          AND bp.parfume_id = $1
        "#,
        id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!("DELETE FROM batch_parfume WHERE parfume_id = $1", id)
        .execute(&mut *tx)
        .await?;

    sqlx::query!("DELETE FROM decant WHERE parfume_id = $1", id)
        .execute(&mut *tx)
        .await?;

    sqlx::query!("DELETE FROM parfume WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok("Berhasil dihapus".to_string())
}

struct ParfumeRankingRow {
    pub parfume_id: Uuid,
    pub parfume_name: String,
    pub brands_name: String,
    pub total_revenue: bigdecimal::BigDecimal,
    pub total_cost: bigdecimal::BigDecimal,
    pub total_orders: i64,
}

pub const RANKING_DEFAULT_PER_PAGE: i64 = 10;
pub const RANKING_MAX_PER_PAGE: i64 = 50;

pub async fn svc_get_parfume_ranking(
    pool: &PgPool,
    access: &AccesClaims,
    query: &RankingQuery,
) -> Result<(Vec<ParfumeRankingPoint>, i64), AppError> {
    let uuid = match Uuid::parse_str(&access.sub) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_get_parfume_ranking: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    let frame = match query.frame.as_deref() {
        None | Some("all") => "all",
        Some("day") | Some("daily") => "day",
        Some("week") | Some("weekly") => "week",
        Some("month") | Some("monthly") => "month",
        Some(_) => {
            return Err(AppError::BadRequest(
                Some("frame harus day, week, month, atau all".to_string()),
                Some("svc_get_parfume_ranking: frame tidak valid".to_string()),
            ));
        }
    };

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(RANKING_DEFAULT_PER_PAGE).clamp(1, RANKING_MAX_PER_PAGE);
    let offset = page.saturating_sub(1).saturating_mul(per_page);

    let now = Utc::now();

    let (start_filter, start_date_next) = match frame {
        "day" => {
            let d = now.date_naive();
            let start = d.and_hms_opt(0, 0, 0).expect("jam 00:00:00 selalu valid").and_utc();
            let end = (d + ChronoDuration::days(1)).and_hms_opt(0, 0, 0).expect("jam 00:00:00 selalu valid").and_utc();
            (Some(start), Some(end))
        }
        "week" => {
            let d = now.date_naive();
            let weekday = d.weekday().num_days_from_monday();
            let week_start = d - ChronoDuration::days(weekday as i64);
            let week_end = week_start + ChronoDuration::days(7);
            (
                Some(week_start.and_hms_opt(0, 0, 0).expect("jam 00:00:00 selalu valid").and_utc()),
                Some(week_end.and_hms_opt(0, 0, 0).expect("jam 00:00:00 selalu valid").and_utc()),
            )
        }
        "month" => {
            let d = now.date_naive();
            let month_start = d.with_day(1).expect("tanggal 1 selalu valid");
            let month_end = month_start + ChronoDuration::days(32);
            let month_end = month_end.with_day(1).expect("tanggal 1 selalu valid");
            (
                Some(month_start.and_hms_opt(0, 0, 0).expect("jam 00:00:00 selalu valid").and_utc()),
                Some(month_end.and_hms_opt(0, 0, 0).expect("jam 00:00:00 selalu valid").and_utc()),
            )
        }
        _ => (None, None),
    };

    let total_items: i64 = match frame {
        "day" | "week" | "month" => {
            sqlx::query_scalar!(
                r#"
                WITH lines AS (
                    SELECT bp.parfume_id AS parfume_id
                    FROM order_items oi
                    JOIN batch_parfume_bottle bf ON bf.id = oi.bottle_id
                    JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
                    JOIN parfume p ON p.id = bp.parfume_id
                    JOIN brands br ON br.id = p.brands_id
                    JOIN decant d ON d.id = oi.decant_id
                    WHERE br.owner_id = $1
                      AND oi.status = 'success'
                      AND oi.created_at >= $2
                      AND oi.created_at < $3
                    GROUP BY bp.parfume_id
                )
                SELECT COUNT(*) AS "count!"
                FROM lines"#,
                uuid,
                start_filter.unwrap(),
                start_date_next.unwrap()
            )
            .fetch_one(pool)
            .await?
        }
        _ => {
            sqlx::query_scalar!(
                r#"
                WITH lines AS (
                    SELECT bp.parfume_id AS parfume_id
                    FROM order_items oi
                    JOIN batch_parfume_bottle bf ON bf.id = oi.bottle_id
                    JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
                    JOIN parfume p ON p.id = bp.parfume_id
                    JOIN brands br ON br.id = p.brands_id
                    JOIN decant d ON d.id = oi.decant_id
                    WHERE br.owner_id = $1
                      AND oi.status = 'success'
                    GROUP BY bp.parfume_id
                )
                SELECT COUNT(*) AS "count!"
                FROM lines"#,
                uuid
            )
            .fetch_one(pool)
            .await?
        }
    };

    let rows: Vec<ParfumeRankingRow> = match frame {
        "day" | "week" | "month" => {
            sqlx::query_as!(
                ParfumeRankingRow,
                r#"
                WITH lines AS (
                    SELECT
                        oi.total_price AS total_price,
                        (bp.purchase_price / NULLIF(bp.quantity_ml, 0)) * d.size_ml * oi.quantity AS cost,
                        bp.parfume_id AS parfume_id,
                        p.name AS parfume_name,
                        br.name AS brands_name
                    FROM order_items oi
                    JOIN batch_parfume_bottle bf ON bf.id = oi.bottle_id
                    JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
                    JOIN parfume p ON p.id = bp.parfume_id
                    JOIN brands br ON br.id = p.brands_id
                    JOIN decant d ON d.id = oi.decant_id
                    WHERE br.owner_id = $1
                      AND oi.status = 'success'
                      AND oi.created_at >= $2
                      AND oi.created_at < $3
                )
                SELECT
                    parfume_id AS "parfume_id!",
                    parfume_name AS "parfume_name!",
                    brands_name AS "brands_name!",
                    COALESCE(SUM(total_price), 0) AS "total_revenue!",
                    COALESCE(SUM(cost), 0) AS "total_cost!",
                    COUNT(*) AS "total_orders!"
                FROM lines
                GROUP BY parfume_id, parfume_name, brands_name
                ORDER BY 4 DESC
                LIMIT $4 OFFSET $5"#,
                uuid,
                start_filter.unwrap(),
                start_date_next.unwrap(),
                per_page,
                offset
            )
            .fetch_all(pool)
            .await?
        }
        _ => {
            sqlx::query_as!(
                ParfumeRankingRow,
                r#"
                WITH lines AS (
                    SELECT
                        oi.total_price AS total_price,
                        (bp.purchase_price / NULLIF(bp.quantity_ml, 0)) * d.size_ml * oi.quantity AS cost,
                        bp.parfume_id AS parfume_id,
                        p.name AS parfume_name,
                        br.name AS brands_name
                    FROM order_items oi
                    JOIN batch_parfume_bottle bf ON bf.id = oi.bottle_id
                    JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
                    JOIN parfume p ON p.id = bp.parfume_id
                    JOIN brands br ON br.id = p.brands_id
                    JOIN decant d ON d.id = oi.decant_id
                    WHERE br.owner_id = $1
                      AND oi.status = 'success'
                )
                SELECT
                    parfume_id AS "parfume_id!",
                    parfume_name AS "parfume_name!",
                    brands_name AS "brands_name!",
                    COALESCE(SUM(total_price), 0) AS "total_revenue!",
                    COALESCE(SUM(cost), 0) AS "total_cost!",
                    COUNT(*) AS "total_orders!"
                FROM lines
                GROUP BY parfume_id, parfume_name, brands_name
                ORDER BY 4 DESC
                LIMIT $2 OFFSET $3"#,
                uuid,
                per_page,
                offset
            )
            .fetch_all(pool)
            .await?
        }
    };

    let points = rows
        .into_iter()
        .enumerate()
        .map(|(i, row)| {
            let estimated_profit = &row.total_revenue - &row.total_cost;
            ParfumeRankingPoint {
                parfume_id: row.parfume_id,
                parfume_name: row.parfume_name,
                brands_name: row.brands_name,
                total_revenue: row.total_revenue,
                total_cost: row.total_cost,
                estimated_profit,
                total_orders: row.total_orders,
                rank: offset + (i as i64) + 1,
            }
        })
        .collect();

    Ok((points, total_items))
}

fn parfume_history_point(
    period: chrono::DateTime<Utc>,
    total_revenue: bigdecimal::BigDecimal,
    total_cost: bigdecimal::BigDecimal,
    total_orders: i64,
) -> RevenueHistoryPoint {
    let estimated_profit = &total_revenue - &total_cost;
    RevenueHistoryPoint {
        period,
        total_revenue,
        total_cost,
        estimated_profit,
        total_orders,
    }
}

pub async fn svc_get_parfume_history(
    pool: &PgPool,
    access: &AccesClaims,
    parfume_id: &Uuid,
    query: &RevenueHistoryQuery,
) -> Result<Vec<RevenueHistoryPoint>, AppError> {
    let owner_uuid = match Uuid::parse_str(&access.sub) {
        Ok(val) => val,
        Err(_) => {
            return Err(AppError::InternalServerError(
                None,
                Some("svc_get_parfume_history: gagal parse UUID dari claims".to_string()),
            ));
        }
    };

    let frame = match query.frame.as_deref() {
        None => HistoryFrame::Day,
        Some(raw) => match HistoryFrame::parse(raw) {
            Some(val) => val,
            None => {
                return Err(AppError::BadRequest(
                    Some("frame harus day, week, month, atau all".to_string()),
                    Some("svc_get_parfume_history: frame harus day, week, month, atau all".to_string()),
                ));
            }
        },
    };

    let anchor = query.date.unwrap_or_else(|| Utc::now().date_naive());

    let parfume_row = sqlx::query!(
        r#"
        SELECT p.id AS "id!"
        FROM parfume p
        JOIN brands b ON b.id = p.brands_id
        WHERE p.id = $1
          AND b.owner_id = $2
          AND p.deleted_at IS NULL
          AND b.deleted_at IS NULL
        "#,
        parfume_id,
        owner_uuid
    )
    .fetch_optional(pool)
    .await?;

    if parfume_row.is_none() {
        return Err(AppError::NotFound(
            None,
            Some("svc_get_parfume_history: parfume tidak ditemukan".to_string()),
        ));
    }

    match frame {
        HistoryFrame::Day => {
            let start_naive: NaiveDateTime = anchor
                .and_hms_opt(0, 0, 0)
                .expect("jam 00:00:00 selalu valid");
            let end_naive = start_naive + ChronoDuration::days(1);

            let rows = sqlx::query!(
                r#"
                WITH lines AS (
                    SELECT
                        oi.created_at AS created_at,
                        oi.total_price AS total_price,
                        (bp.purchase_price / NULLIF(bp.quantity_ml, 0)) * d.size_ml * oi.quantity AS cost
                    FROM order_items oi
                    JOIN batch_parfume_bottle bf ON bf.id = oi.bottle_id
                    JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
                    JOIN parfume p ON p.id = bp.parfume_id
                    JOIN brands br ON br.id = p.brands_id
                    JOIN decant d ON d.id = oi.decant_id
                    WHERE br.owner_id = $1
                      AND p.id = $2
                      AND oi.status = 'success'
                      AND oi.created_at >= $3
                      AND oi.created_at < $4
                )
                SELECT
                    date_trunc('hour', created_at) AS "period!",
                    COALESCE(SUM(total_price), 0) AS "total_revenue!",
                    COALESCE(SUM(cost), 0) AS "total_cost!",
                    COUNT(*) AS "total_orders!"
                FROM lines
                GROUP BY 1
                ORDER BY 1"#,
                owner_uuid,
                parfume_id,
                start_naive.and_utc(),
                end_naive.and_utc()
            )
            .fetch_all(pool)
            .await?;

            let mut by_hour: HashMap<chrono::DateTime<Utc>, (bigdecimal::BigDecimal, bigdecimal::BigDecimal, i64)> =
                HashMap::with_capacity(rows.len());
            for row in rows {
                by_hour.insert(row.period, (row.total_revenue, row.total_cost, row.total_orders));
            }

            let mut points = Vec::with_capacity(24);
            for hour in 0..24 {
                let period = (start_naive + ChronoDuration::hours(hour)).and_utc();
                match by_hour.remove(&period) {
                    Some((r, c, o)) => points.push(parfume_history_point(period, r, c, o)),
                    None => points.push(parfume_history_point(period, bigdecimal::BigDecimal::from(0), bigdecimal::BigDecimal::from(0), 0)),
                }
            }

            Ok(points)
        }
        HistoryFrame::Week | HistoryFrame::Month => {
            let start_date = if frame == HistoryFrame::Week {
                anchor - ChronoDuration::days(anchor.weekday().num_days_from_monday() as i64)
            } else {
                anchor.with_day(1).expect("tanggal 1 selalu valid")
            };
            let end_date = if frame == HistoryFrame::Week {
                start_date + ChronoDuration::days(7)
            } else {
                start_date
                    .checked_add_months(Months::new(1))
                    .expect("bulan berikutnya selalu valid")
            };

            let rows = sqlx::query!(
                r#"
                WITH lines AS (
                    SELECT
                        oi.created_at AS created_at,
                        oi.total_price AS total_price,
                        (bp.purchase_price / NULLIF(bp.quantity_ml, 0)) * d.size_ml * oi.quantity AS cost
                    FROM order_items oi
                    JOIN batch_parfume_bottle bf ON bf.id = oi.bottle_id
                    JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
                    JOIN parfume p ON p.id = bp.parfume_id
                    JOIN brands br ON br.id = p.brands_id
                    JOIN decant d ON d.id = oi.decant_id
                    WHERE br.owner_id = $1
                      AND p.id = $2
                      AND oi.status = 'success'
                      AND oi.created_at >= $3
                      AND oi.created_at < $4
                )
                SELECT
                    date_trunc('day', created_at)::date AS "period!",
                    COALESCE(SUM(total_price), 0) AS "total_revenue!",
                    COALESCE(SUM(cost), 0) AS "total_cost!",
                    COUNT(*) AS "total_orders!"
                FROM lines
                GROUP BY 1
                ORDER BY 1"#,
                owner_uuid,
                parfume_id,
                start_date.and_hms_opt(0, 0, 0).expect("jam 00:00:00 selalu valid").and_utc(),
                end_date.and_hms_opt(0, 0, 0).expect("jam 00:00:00 selalu valid").and_utc()
            )
            .fetch_all(pool)
            .await?;

            let mut by_day: HashMap<NaiveDate, (bigdecimal::BigDecimal, bigdecimal::BigDecimal, i64)> =
                HashMap::with_capacity(rows.len());
            for row in rows {
                by_day.insert(row.period, (row.total_revenue, row.total_cost, row.total_orders));
            }

            let mut points = Vec::new();
            let mut cursor = start_date;
            while cursor < end_date {
                let period = cursor.and_hms_opt(0, 0, 0).expect("jam 00:00:00 selalu valid").and_utc();
                match by_day.remove(&cursor) {
                    Some((r, c, o)) => points.push(parfume_history_point(period, r, c, o)),
                    None => points.push(parfume_history_point(period, bigdecimal::BigDecimal::from(0), bigdecimal::BigDecimal::from(0), 0)),
                }
                cursor = cursor + ChronoDuration::days(1);
            }

            Ok(points)
        }
        HistoryFrame::All => {
            let rows = sqlx::query!(
                r#"
                WITH lines AS (
                    SELECT
                        oi.created_at AS created_at,
                        oi.total_price AS total_price,
                        (bp.purchase_price / NULLIF(bp.quantity_ml, 0)) * d.size_ml * oi.quantity AS cost
                    FROM order_items oi
                    JOIN batch_parfume_bottle bf ON bf.id = oi.bottle_id
                    JOIN batch_parfume bp ON bp.id = bf.batch_parfume_id
                    JOIN parfume p ON p.id = bp.parfume_id
                    JOIN brands br ON br.id = p.brands_id
                    JOIN decant d ON d.id = oi.decant_id
                    WHERE br.owner_id = $1
                      AND p.id = $2
                      AND oi.status = 'success'
                )
                SELECT
                    date_trunc('month', created_at)::date AS "period!",
                    COALESCE(SUM(total_price), 0) AS "total_revenue!",
                    COALESCE(SUM(cost), 0) AS "total_cost!",
                    COUNT(*) AS "total_orders!"
                FROM lines
                GROUP BY 1
                ORDER BY 1"#,
                owner_uuid,
                parfume_id
            )
            .fetch_all(pool)
            .await?;

            if rows.is_empty() {
                return Ok(vec![]);
            }

            let mut by_month: HashMap<i32, (bigdecimal::BigDecimal, bigdecimal::BigDecimal, i64)> =
                HashMap::with_capacity(rows.len());
            for row in rows {
                let mi = row.period.year() * 12 + row.period.month() as i32 - 1;
                by_month.insert(mi, (row.total_revenue, row.total_cost, row.total_orders));
            }

            let current_month_index = {
                let cur = Utc::now().date_naive().with_day(1).expect("tanggal 1 selalu valid");
                cur.year() * 12 + cur.month() as i32 - 1
            };
            let first_month_index = by_month.keys().copied().min().expect("rows tidak kosong");

            let mut points = Vec::with_capacity((current_month_index - first_month_index + 1).max(0) as usize);
            for index in first_month_index..=current_month_index {
                let year = index.div_euclid(12);
                let month = (index.rem_euclid(12) + 1) as u32;
                let period_date = NaiveDate::from_ymd_opt(year, month, 1).expect("bulan valid");
                let period = period_date.and_hms_opt(0, 0, 0).expect("jam 00:00:00 selalu valid").and_utc();
                match by_month.remove(&index) {
                    Some((r, c, o)) => points.push(parfume_history_point(period, r, c, o)),
                    None => points.push(parfume_history_point(period, bigdecimal::BigDecimal::from(0), bigdecimal::BigDecimal::from(0), 0)),
                }
            }

            Ok(points)
        }
    }
}
