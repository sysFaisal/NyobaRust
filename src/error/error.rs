use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use sqlx;

/// Slot pesan yang dikirim ke client (muncul di response JSON).
pub type ClientMsg = Option<String>;
/// Slot pesan yang hanya muncul di log internal (untuk debugging).
pub type LogMsg = Option<String>;

///
/// Cara pakai:
/// - `AppError::BadRequest(None, None)` -> default, tanpa detail
/// - `AppError::BadRequest(Some(msg), None)` -> pesan hanya ke client
/// - `AppError::BadRequest(None, Some(ctx))` -> konteks hanya ke log
/// - `AppError::BadRequest(Some(msg), Some(ctx))` -> ke client DAN log
///
/// Contoh:
/// `AppError::BadRequest(Some(e), Some("bad req di fn create_user"))`
#[derive(Debug)]
pub enum AppError {
    /// (pesan_client, pesan_log)
    NotFound(ClientMsg, LogMsg),
    /// (pesan_client, pesan_log)
    Unauthorized(ClientMsg, LogMsg),
    /// (pesan_client, pesan_log)
    Forbidden(ClientMsg, LogMsg),
    /// (pesan_client, pesan_log)
    Conflict(ClientMsg, LogMsg),
    /// (pesan_client, pesan_log)
    InternalServerError(ClientMsg, LogMsg),
    /// (error_sqlx, pesan_log)
    Database(sqlx::Error, LogMsg),
    /// (pesan_client, pesan_log)
    BadRequest(ClientMsg, LogMsg),
}

#[derive(Serialize)]
pub struct ErrMsgClient {
    code: String,
    message: String,
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err, None)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, code, fallback, client_msg, log_msg) = match self {
            AppError::Database(err, log) => {
                tracing::error!(
                    error_code = "DATABASE_ERROR",
                    error = ?err,
                    context = ?log,
                    "AppError"
                );
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrMsgClient {
                        code: "INTERNAL_SERVER_ERROR".to_string(),
                        message: "Internal Server Error".to_string(),
                    }),
                )
                    .into_response();
            }

            AppError::InternalServerError(client, log) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "Internal Server Error",
                client,
                log,
            ),

            AppError::NotFound(client, log) => {
                (StatusCode::NOT_FOUND, "NOT_FOUND", "Not Found", client, log)
            }

            AppError::BadRequest(client, log) => (
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST",
                "Bad Request",
                client,
                log,
            ),

            AppError::Unauthorized(client, log) => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Authentication required",
                client,
                log,
            ),

            AppError::Forbidden(client, log) => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "You do not have permission",
                client,
                log,
            ),

            AppError::Conflict(client, log) => (
                StatusCode::CONFLICT,
                "CONFLICT",
                "Resource conflict",
                client,
                log,
            ),
        };

        // Error 5xx selalu dicatat sebagai error, 4xx sebagai warning.
        if status.is_server_error() {
            tracing::error!(
                status = %status,
                error_code = %code,
                client_message = ?client_msg,
                context = ?log_msg,
                "AppError"
            );
        } else {
            tracing::warn!(
                status = %status,
                error_code = %code,
                client_message = ?client_msg,
                context = ?log_msg,
                "AppError"
            );
        }

        let message = client_msg.unwrap_or_else(|| fallback.to_string());

        (
            status,
            Json(ErrMsgClient {
                code: code.to_string(),
                message,
            }),
        )
            .into_response()
    }
}
