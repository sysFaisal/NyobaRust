use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Granularity {
    Day,
    Week,
    Month,
}

impl Granularity {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "day" => Some(Granularity::Day),
            "week" => Some(Granularity::Week),
            "month" => Some(Granularity::Month),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RevenueBucket {
    pub total_revenue: BigDecimal,
    pub total_cost: BigDecimal,
    pub estimated_profit: BigDecimal,
    pub total_orders: i64,
}

impl RevenueBucket {
    pub fn new(total_revenue: BigDecimal, total_cost: BigDecimal, total_orders: i64) -> Self {
        let estimated_profit = &total_revenue - &total_cost;
        Self {
            total_revenue,
            total_cost,
            estimated_profit,
            total_orders,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RevenueSummaryResponse {
    pub today: RevenueBucket,
    pub week: RevenueBucket,
    pub month: RevenueBucket,
    pub all_time: RevenueBucket,
}

#[derive(Debug, Serialize)]
pub struct DailyRevenuePoint {
    pub period: NaiveDate,
    pub total_revenue: BigDecimal,
    pub total_cost: BigDecimal,
    pub estimated_profit: BigDecimal,
    pub total_orders: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HistoryFrame {
    Day,
    Week,
    Month,
    All,
}

impl HistoryFrame {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "day" | "daily" => Some(HistoryFrame::Day),
            "week" | "weekly" => Some(HistoryFrame::Week),
            "month" | "monthly" => Some(HistoryFrame::Month),
            "all" | "all_time" => Some(HistoryFrame::All),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RevenueHistoryPoint {
    pub period: DateTime<Utc>,
    pub total_revenue: BigDecimal,
    pub total_cost: BigDecimal,
    pub estimated_profit: BigDecimal,
    pub total_orders: i64,
}
