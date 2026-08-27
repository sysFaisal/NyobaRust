use bigdecimal::BigDecimal;
use chrono::{
    DateTime, Datelike, Duration as ChronoDuration, Months, NaiveDate, NaiveDateTime, Utc,
};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    c_auth::refresh_token::AccesClaims,
    dto::{
        request::revenue_req::{DailyRevenueQuery, RevenueHistoryQuery},
        response::revenue_res::{
            DailyRevenuePoint, Granularity, HistoryFrame, RevenueBucket, RevenueHistoryPoint,
            RevenueSummaryResponse,
        },
    },
    error::error::AppError,
};

pub const DAILY_REVENUE_DEFAULT_DAYS: i64 = 30;
pub const REVENUE_MAX_RANGE_DAYS: i64 = 366;

fn parse_owner_uuid(context: &str, access: &AccesClaims) -> Result<Uuid, AppError> {
    match Uuid::parse_str(&access.sub) {
        Ok(val) => Ok(val),
        Err(_) => Err(AppError::InternalServerError(
            None,
            Some(format!("{context}: gagal parse UUID dari claims")),
        )),
    }
}

pub async fn svc_get_revenue_summary(
    pool: &PgPool,
    access: &AccesClaims,
) -> Result<RevenueSummaryResponse, AppError> {
    let uuid = parse_owner_uuid("svc_get_revenue_summary", access)?;

    let row = sqlx::query!(
        r#"
        WITH lines AS (
            SELECT
                oi.created_at AS created_at,
                oi.total_price AS total_price,
                (bp.purchase_price / NULLIF(bp.quantity_ml, 0)) * d.size_ml * oi.quantity AS cost
            FROM order_items oi
            JOIN batch_parfume_bottle bf
                ON bf.id = oi.bottle_id
            JOIN batch_parfume bp
                ON bp.id = bf.batch_parfume_id
            JOIN parfume p
                ON p.id = bp.parfume_id
            JOIN brands br
                ON br.id = p.brands_id
            JOIN decant d
                ON d.id = oi.decant_id
            WHERE br.owner_id = $1
              AND oi.status = 'success'
        )
        SELECT
            COALESCE(SUM(total_price) FILTER (
                WHERE created_at >= date_trunc('day', now())
            ), 0) AS "today_revenue!",
            COALESCE(SUM(cost) FILTER (
                WHERE created_at >= date_trunc('day', now())
            ), 0) AS "today_cost!",
            COUNT(*) FILTER (
                WHERE created_at >= date_trunc('day', now())
            ) AS "today_orders!",
            COALESCE(SUM(total_price) FILTER (
                WHERE created_at >= date_trunc('week', now())
            ), 0) AS "week_revenue!",
            COALESCE(SUM(cost) FILTER (
                WHERE created_at >= date_trunc('week', now())
            ), 0) AS "week_cost!",
            COUNT(*) FILTER (
                WHERE created_at >= date_trunc('week', now())
            ) AS "week_orders!",
            COALESCE(SUM(total_price) FILTER (
                WHERE created_at >= date_trunc('month', now())
            ), 0) AS "month_revenue!",
            COALESCE(SUM(cost) FILTER (
                WHERE created_at >= date_trunc('month', now())
            ), 0) AS "month_cost!",
            COUNT(*) FILTER (
                WHERE created_at >= date_trunc('month', now())
            ) AS "month_orders!",
            COALESCE(SUM(total_price), 0) AS "all_time_revenue!",
            COALESCE(SUM(cost), 0) AS "all_time_cost!",
            COUNT(*) AS "all_time_orders!"
        FROM lines"#,
        uuid
    )
    .fetch_one(pool)
    .await?;

    Ok(RevenueSummaryResponse {
        today: RevenueBucket::new(row.today_revenue, row.today_cost, row.today_orders),
        week: RevenueBucket::new(row.week_revenue, row.week_cost, row.week_orders),
        month: RevenueBucket::new(row.month_revenue, row.month_cost, row.month_orders),
        all_time: RevenueBucket::new(row.all_time_revenue, row.all_time_cost, row.all_time_orders),
    })
}

pub async fn svc_get_daily_revenue(
    pool: &PgPool,
    access: &AccesClaims,
    query: &DailyRevenueQuery,
) -> Result<Vec<DailyRevenuePoint>, AppError> {
    let uuid = parse_owner_uuid("svc_get_daily_revenue", access)?;

    let to = query.to.unwrap_or_else(|| Utc::now().date_naive());
    let from = query
        .from
        .unwrap_or_else(|| to - ChronoDuration::days(DAILY_REVENUE_DEFAULT_DAYS - 1));

    if from > to {
        return Err(AppError::BadRequest(
            Some("tanggal 'from' tidak boleh lebih besar dari 'to'".to_string()),
            Some(
                "svc_get_daily_revenue: tanggal 'from' tidak boleh lebih besar dari 'to'"
                    .to_string(),
            ),
        ));
    }

    if (to - from).num_days() + 1 > REVENUE_MAX_RANGE_DAYS {
        return Err(AppError::BadRequest(
            Some(format!("rentang maksimal {} hari", REVENUE_MAX_RANGE_DAYS)),
            Some(format!(
                "svc_get_daily_revenue: rentang maksimal {} hari",
                REVENUE_MAX_RANGE_DAYS
            )),
        ));
    }

    let granularity = match query.granularity.as_deref() {
        None => Granularity::Day,
        Some(raw) => match Granularity::parse(raw) {
            Some(val) => val,
            None => {
                return Err(AppError::BadRequest(
                    Some("granularity harus day, week, atau month".to_string()),
                    Some(
                        "svc_get_daily_revenue: granularity harus day, week, atau month"
                            .to_string(),
                    ),
                ));
            }
        },
    };

    let start = from
        .and_hms_opt(0, 0, 0)
        .expect("jam 00:00:00 selalu valid")
        .and_utc();
    let end_date = to + ChronoDuration::days(1);
    let end = end_date
        .and_hms_opt(0, 0, 0)
        .expect("jam 00:00:00 selalu valid")
        .and_utc();

    let rows: Vec<RevenueSeriesRow> = match granularity {
        Granularity::Day => {
            sqlx::query_as!(
                RevenueSeriesRow,
                r#"
                WITH lines AS (
                    SELECT
                        oi.created_at AS created_at,
                        oi.total_price AS total_price,
                        (bp.purchase_price / NULLIF(bp.quantity_ml, 0)) * d.size_ml * oi.quantity AS cost
                    FROM order_items oi
                    JOIN batch_parfume_bottle bf
                        ON bf.id = oi.bottle_id
                    JOIN batch_parfume bp
                        ON bp.id = bf.batch_parfume_id
                    JOIN parfume p
                        ON p.id = bp.parfume_id
                    JOIN brands br
                        ON br.id = p.brands_id
                    JOIN decant d
                        ON d.id = oi.decant_id
                    WHERE br.owner_id = $1
                      AND oi.status = 'success'
                      AND oi.created_at >= $2
                      AND oi.created_at < $3
                )
                SELECT
                    date_trunc('day', created_at)::date AS "period!",
                    COALESCE(SUM(total_price), 0) AS "total_revenue!",
                    COALESCE(SUM(cost), 0) AS "total_cost!",
                    COUNT(*) AS "total_orders!"
                FROM lines
                GROUP BY 1
                ORDER BY 1"#,
                uuid,
                start,
                end
            )
            .fetch_all(pool)
            .await?
        }
        Granularity::Week => {
            sqlx::query_as!(
                RevenueSeriesRow,
                r#"
                WITH lines AS (
                    SELECT
                        oi.created_at AS created_at,
                        oi.total_price AS total_price,
                        (bp.purchase_price / NULLIF(bp.quantity_ml, 0)) * d.size_ml * oi.quantity AS cost
                    FROM order_items oi
                    JOIN batch_parfume_bottle bf
                        ON bf.id = oi.bottle_id
                    JOIN batch_parfume bp
                        ON bp.id = bf.batch_parfume_id
                    JOIN parfume p
                        ON p.id = bp.parfume_id
                    JOIN brands br
                        ON br.id = p.brands_id
                    JOIN decant d
                        ON d.id = oi.decant_id
                    WHERE br.owner_id = $1
                      AND oi.status = 'success'
                      AND oi.created_at >= $2
                      AND oi.created_at < $3
                )
                SELECT
                    date_trunc('week', created_at)::date AS "period!",
                    COALESCE(SUM(total_price), 0) AS "total_revenue!",
                    COALESCE(SUM(cost), 0) AS "total_cost!",
                    COUNT(*) AS "total_orders!"
                FROM lines
                GROUP BY 1
                ORDER BY 1"#,
                uuid,
                start,
                end
            )
            .fetch_all(pool)
            .await?
        }
        Granularity::Month => {
            sqlx::query_as!(
                RevenueSeriesRow,
                r#"
                WITH lines AS (
                    SELECT
                        oi.created_at AS created_at,
                        oi.total_price AS total_price,
                        (bp.purchase_price / NULLIF(bp.quantity_ml, 0)) * d.size_ml * oi.quantity AS cost
                    FROM order_items oi
                    JOIN batch_parfume_bottle bf
                        ON bf.id = oi.bottle_id
                    JOIN batch_parfume bp
                        ON bp.id = bf.batch_parfume_id
                    JOIN parfume p
                        ON p.id = bp.parfume_id
                    JOIN brands br
                        ON br.id = p.brands_id
                    JOIN decant d
                        ON d.id = oi.decant_id
                    WHERE br.owner_id = $1
                      AND oi.status = 'success'
                      AND oi.created_at >= $2
                      AND oi.created_at < $3
                )
                SELECT
                    date_trunc('month', created_at)::date AS "period!",
                    COALESCE(SUM(total_price), 0) AS "total_revenue!",
                    COALESCE(SUM(cost), 0) AS "total_cost!",
                    COUNT(*) AS "total_orders!"
                FROM lines
                GROUP BY 1
                ORDER BY 1"#,
                uuid,
                start,
                end
            )
            .fetch_all(pool)
            .await?
        }
    };

    Ok(rows
        .into_iter()
        .map(|row| DailyRevenuePoint {
            period: row.period,
            estimated_profit: &row.total_revenue - &row.total_cost,
            total_revenue: row.total_revenue,
            total_cost: row.total_cost,
            total_orders: row.total_orders,
        })
        .collect())
}

struct RevenueSeriesRow {
    pub period: NaiveDate,
    pub total_revenue: BigDecimal,
    pub total_cost: BigDecimal,
    pub total_orders: i64,
}

struct RevenueHourlyRow {
    pub period: DateTime<Utc>,
    pub total_revenue: BigDecimal,
    pub total_cost: BigDecimal,
    pub total_orders: i64,
}

fn history_point(
    period: DateTime<Utc>,
    total_revenue: BigDecimal,
    total_cost: BigDecimal,
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

fn zero_point(period: DateTime<Utc>) -> RevenueHistoryPoint {
    history_point(period, BigDecimal::from(0), BigDecimal::from(0), 0)
}

fn month_index(date: NaiveDate) -> i32 {
    date.year() * 12 + date.month() as i32 - 1
}

fn date_from_month_index(index: i32) -> NaiveDate {
    NaiveDate::from_ymd_opt(index.div_euclid(12), (index.rem_euclid(12) + 1) as u32, 1)
        .expect("bulan hasil index selalu valid")
}

pub async fn svc_get_revenue_history(
    pool: &PgPool,
    access: &AccesClaims,
    query: &RevenueHistoryQuery,
) -> Result<Vec<RevenueHistoryPoint>, AppError> {
    let uuid = parse_owner_uuid("svc_get_revenue_history", access)?;

    let frame = match query.frame.as_deref() {
        None => HistoryFrame::Day,
        Some(raw) => match HistoryFrame::parse(raw) {
            Some(val) => val,
            None => {
                return Err(AppError::BadRequest(
                    Some("frame harus day, week, month, atau all".to_string()),
                    Some(
                        "svc_get_revenue_history: frame harus day, week, month, atau all"
                            .to_string(),
                    ),
                ));
            }
        },
    };

    let anchor = query.date.unwrap_or_else(|| Utc::now().date_naive());

    match frame {
        HistoryFrame::Day => {
            let start_naive: NaiveDateTime = anchor
                .and_hms_opt(0, 0, 0)
                .expect("jam 00:00:00 selalu valid");
            let end_naive = start_naive + ChronoDuration::days(1);

            let rows = sqlx::query_as!(
                RevenueHourlyRow,
                r#"
                WITH lines AS (
                    SELECT
                        oi.created_at AS created_at,
                        oi.total_price AS total_price,
                        (bp.purchase_price / NULLIF(bp.quantity_ml, 0)) * d.size_ml * oi.quantity AS cost
                    FROM order_items oi
                    JOIN batch_parfume_bottle bf
                        ON bf.id = oi.bottle_id
                    JOIN batch_parfume bp
                        ON bp.id = bf.batch_parfume_id
                    JOIN parfume p
                        ON p.id = bp.parfume_id
                    JOIN brands br
                        ON br.id = p.brands_id
                    JOIN decant d
                        ON d.id = oi.decant_id
                    WHERE br.owner_id = $1
                      AND oi.status = 'success'
                      AND oi.created_at >= $2
                      AND oi.created_at < $3
                )
                SELECT
                    date_trunc('hour', created_at) AS "period!",
                    COALESCE(SUM(total_price), 0) AS "total_revenue!",
                    COALESCE(SUM(cost), 0) AS "total_cost!",
                    COUNT(*) AS "total_orders!"
                FROM lines
                GROUP BY 1
                ORDER BY 1"#,
                uuid,
                start_naive.and_utc(),
                end_naive.and_utc()
            )
            .fetch_all(pool)
            .await?;

            let mut by_hour: HashMap<DateTime<Utc>, (BigDecimal, BigDecimal, i64)> =
                HashMap::with_capacity(rows.len());
            for row in rows {
                by_hour.insert(
                    row.period,
                    (row.total_revenue, row.total_cost, row.total_orders),
                );
            }

            let mut points = Vec::with_capacity(24);
            for hour in 0..24 {
                let period = (start_naive + ChronoDuration::hours(hour)).and_utc();
                match by_hour.remove(&period) {
                    Some((revenue, cost, orders)) => {
                        points.push(history_point(period, revenue, cost, orders))
                    }
                    None => points.push(zero_point(period)),
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

            let rows = sqlx::query_as!(
                RevenueSeriesRow,
                r#"
                WITH lines AS (
                    SELECT
                        oi.created_at AS created_at,
                        oi.total_price AS total_price,
                        (bp.purchase_price / NULLIF(bp.quantity_ml, 0)) * d.size_ml * oi.quantity AS cost
                    FROM order_items oi
                    JOIN batch_parfume_bottle bf
                        ON bf.id = oi.bottle_id
                    JOIN batch_parfume bp
                        ON bp.id = bf.batch_parfume_id
                    JOIN parfume p
                        ON p.id = bp.parfume_id
                    JOIN brands br
                        ON br.id = p.brands_id
                    JOIN decant d
                        ON d.id = oi.decant_id
                    WHERE br.owner_id = $1
                      AND oi.status = 'success'
                      AND oi.created_at >= $2
                      AND oi.created_at < $3
                )
                SELECT
                    date_trunc('day', created_at)::date AS "period!",
                    COALESCE(SUM(total_price), 0) AS "total_revenue!",
                    COALESCE(SUM(cost), 0) AS "total_cost!",
                    COUNT(*) AS "total_orders!"
                FROM lines
                GROUP BY 1
                ORDER BY 1"#,
                uuid,
                start_date
                    .and_hms_opt(0, 0, 0)
                    .expect("jam 00:00:00 selalu valid")
                    .and_utc(),
                end_date
                    .and_hms_opt(0, 0, 0)
                    .expect("jam 00:00:00 selalu valid")
                    .and_utc()
            )
            .fetch_all(pool)
            .await?;

            let mut by_day: HashMap<NaiveDate, (BigDecimal, BigDecimal, i64)> =
                HashMap::with_capacity(rows.len());
            for row in rows {
                by_day.insert(
                    row.period,
                    (row.total_revenue, row.total_cost, row.total_orders),
                );
            }

            let mut points = Vec::new();
            let mut cursor = start_date;
            while cursor < end_date {
                let period = cursor
                    .and_hms_opt(0, 0, 0)
                    .expect("jam 00:00:00 selalu valid")
                    .and_utc();
                match by_day.remove(&cursor) {
                    Some((revenue, cost, orders)) => {
                        points.push(history_point(period, revenue, cost, orders))
                    }
                    None => points.push(zero_point(period)),
                }
                cursor = cursor + ChronoDuration::days(1);
            }

            Ok(points)
        }
        HistoryFrame::All => {
            let rows = sqlx::query_as!(
                RevenueSeriesRow,
                r#"
                WITH lines AS (
                    SELECT
                        oi.created_at AS created_at,
                        oi.total_price AS total_price,
                        (bp.purchase_price / NULLIF(bp.quantity_ml, 0)) * d.size_ml * oi.quantity AS cost
                    FROM order_items oi
                    JOIN batch_parfume_bottle bf
                        ON bf.id = oi.bottle_id
                    JOIN batch_parfume bp
                        ON bp.id = bf.batch_parfume_id
                    JOIN parfume p
                        ON p.id = bp.parfume_id
                    JOIN brands br
                        ON br.id = p.brands_id
                    JOIN decant d
                        ON d.id = oi.decant_id
                    WHERE br.owner_id = $1
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
                uuid
            )
            .fetch_all(pool)
            .await?;

            if rows.is_empty() {
                return Ok(vec![]);
            }

            let mut by_month: HashMap<i32, (BigDecimal, BigDecimal, i64)> =
                HashMap::with_capacity(rows.len());
            for row in rows {
                by_month.insert(
                    month_index(row.period),
                    (row.total_revenue, row.total_cost, row.total_orders),
                );
            }

            let current_month_index = month_index(
                Utc::now()
                    .date_naive()
                    .with_day(1)
                    .expect("tanggal 1 selalu valid"),
            );
            let first_month_index = by_month.keys().copied().min().expect("rows tidak kosong");

            let mut points =
                Vec::with_capacity((current_month_index - first_month_index + 1).max(0) as usize);
            for index in first_month_index..=current_month_index {
                let period = date_from_month_index(index)
                    .and_hms_opt(0, 0, 0)
                    .expect("jam 00:00:00 selalu valid")
                    .and_utc();
                match by_month.remove(&index) {
                    Some((revenue, cost, orders)) => {
                        points.push(history_point(period, revenue, cost, orders))
                    }
                    None => points.push(zero_point(period)),
                }
            }

            Ok(points)
        }
    }
}
