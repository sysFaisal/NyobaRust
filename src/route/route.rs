use axum::{Router, routing::get};
use sqlx::PgPool;
use crate::handlers::user;

async fn hello() -> &'static str {
    "Hello World"
}

pub fn route_user() -> Router<PgPool> {
    Router::new()
        .route("/", get(user::get_all_user).post(user::create_user))
        .route(
            "/{id}",
            get(user::get_user_by_id)
                .put(hello)
                .patch(hello)
                .delete(user::delete_data_user),
        )
}

pub async fn create_route(pool: PgPool) -> Router {
    Router::new()
        .nest("/api/v1/users", route_user())
        .with_state(pool)
}
