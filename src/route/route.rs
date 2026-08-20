use crate::handlers::brand::create_brands;
use crate::handlers::user::{
    create_user, delete_data_user, get_all_user, get_user_by_id, login_user, refresh_token,
};
use crate::service::service_user::auth_middleware;
use crate::state::AppState;
use axum::middleware;
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
        .route("/refresh", post(refresh_token))
}

pub fn route_user_protected() -> Router<AppState> {
    Router::new()
        .route(
            "/profile/{id}",
            get(get_user_by_id).delete(delete_data_user),
        )
        .layer(middleware::from_fn(auth_middleware))
}

pub fn route_user() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_user))
        .route("/{id}", get(get_user_by_id).delete(delete_data_user))
        .layer(middleware::from_fn(auth_middleware))
}

pub fn router_brands() -> Router<AppState> {
    Router::new()
        .route("/", get(hello).post(create_brands))
        .route("/{id}", get(hello))
        .layer(middleware::from_fn(auth_middleware))
}

pub fn router_parfume() -> Router<AppState> {
    Router::new()
        .route("/", post(create_parfume))
        .layer(middleware::from_fn(auth_middleware))
}

pub async fn create_route(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1/auth", auth_user())
        .nest("/api/v1/users", route_user())
        .nest("/api/v1/brands", router_brands())
        .with_state(state)
}
