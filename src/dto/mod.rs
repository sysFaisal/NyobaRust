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
