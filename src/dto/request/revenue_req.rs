use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DailyRevenueQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub granularity: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RevenueHistoryQuery {
    pub frame: Option<String>,
    pub date: Option<NaiveDate>,
}
