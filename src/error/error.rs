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
    code : String,
    message : String,
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
                        code : "INTERNAL_SERVER_ERROR".to_string(),
                        message : "Internal Server Error".to_string(),
                    }),
                )
                    .into_response()
            }

            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrMsgClient {
                    code : "NOT_FOUND".to_string(),
                    message: "Not Found".to_string(),
                }),
            )
                .into_response(),

            AppError::BadRequest(e_msg) => (
                StatusCode::BAD_REQUEST,
                Json(ErrMsgClient{
                    code : "BAD_REQUEST".to_string(),
                    message : match e_msg {
                        Some(msg) => msg.to_string(),
                        None => "Bad Request".to_string(),
                    },
                }),
            ).into_response(),

            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(ErrMsgClient{
                    code : "UNAUTHORIZED".to_string(),
                    message : "Authentication required".to_string(),
                })
            ).into_response(),

            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                Json(ErrMsgClient{
                    code : "FORBIDDEN".to_string(),
                    message : "You do not have permission".to_string(),
                })
            ).into_response(),

            
            AppError::Conflict => (
                StatusCode::CONFLICT,
                Json(ErrMsgClient {
                    code: "CONFLICT".to_string(),
                    message: "Resource conflict".to_string(),
                }),
            ).into_response(),
        }
    }
}
