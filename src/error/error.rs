use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use sqlx;

pub enum AppError {
    NotFound,
    Unauthorized,
    Forbidden,
    Conflict,
    Database(sqlx::Error),
    BadRequest(Option<String>)
}

#[derive(Serialize)]
pub struct ErrMsgClient {
    error: String,
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::Database(err) => {
                tracing::error!("Database Error : {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrMsgClient {
                        error: "Internal Server Error".to_string(),
                    }),
                )
                    .into_response()
            }

            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrMsgClient {
                    error: "Not Found".to_string(),
                }),
            )
                .into_response(),

            AppError::BadRequest(Some(e_msg)) => (
                StatusCode::BAD_REQUEST,
                Json(ErrMsgClient{
                    error: e_msg,
                }),
            ).into_response()
        }
    }
}
