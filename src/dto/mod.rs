use serde::{Deserialize, Serialize};

pub mod request;
pub mod response;

//Rust Struct ──serialize──> JSON
//Rust Struct <──deserialize── JSON
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub message: Option<String>,
}

#[derive(Serialize)]
pub struct PaginationMeta {
    pub page: i64,
    pub per_page: i64,
    pub total_items: i64,
    pub total_pages: i64,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
    pub message: Option<String>,
}
