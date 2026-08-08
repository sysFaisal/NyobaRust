
use sqlx::PgPool;

use crate::dto::response::response_user::UserProfile;
use crate::error::error::AppError;

pub async fn svc_get_all_user(pool: &PgPool) -> Result<Vec<UserProfile>, AppError> {
    let users = sqlx::query_as!(
        UserProfile,
        "SELECT id, username, email, created_at FROM users"
    )
    .fetch_all(pool)
    .await?;

    Ok(users)
}
