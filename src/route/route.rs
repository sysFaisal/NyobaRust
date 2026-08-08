use axum::{Router, routing::get};
use sqlx::PgPool;

use crate::handlers::user;

async fn hello() -> &'static str {
    "Hello World"
}

pub fn route_user() -> Router<PgPool> {
    Router::new()
        .route("/", get(user::get_all_user).post(hello))
        .route(
            "/{id}",
            get(hello).put(hello).patch(hello).delete(hello),
        )
}

pub async fn create_route(pool: PgPool) -> Router {
    Router::new()
        .nest("/api/v1/users", route_user())
        .with_state(pool)
}
