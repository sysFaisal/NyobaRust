use crate::handlers::user::{
    create_user, delete_data_user, get_all_user, get_user_by_id, login_user,
};
use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

async fn hello() -> &'static str {
    "Hello World"
}

pub fn auth_user() -> Router<AppState> {
    Router::new()
        .route("/register", post(create_user))
        .route("/login", post(login_user))
}

pub fn route_user() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_user).post(create_user))
        .route(
            "/{id}",
            get(get_user_by_id)
                .put(hello)
                .patch(hello)
                .delete(delete_data_user),
        )
}

pub async fn create_route(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1/users", route_user())
        .nest("/api/v1/auth", auth_user())
        .with_state(state)
}
