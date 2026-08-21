use crate::handlers::batch::{create_batch, get_all_batch, update_batch};
use crate::handlers::bottle::create_bottle;
use crate::handlers::brand::{create_brands, get_all_brands, get_brands_by_id, update_brands};
use crate::handlers::decant::{create_decant, get_all_decant};
use crate::handlers::parfume::create_parfum;
use crate::handlers::user::{
    create_user, delete_data_user, get_all_user, get_user_by_id, login_user, refresh_token,
    update_user,
};
use crate::service::brands_svc::svc_get_all_brands;
use crate::service::user_svc::auth_middleware;
use crate::state::AppState;
use axum::middleware;
use axum::routing::patch;
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
        .route(
            "/{id}",
            get(get_user_by_id)
                .delete(delete_data_user)
                .patch(update_user),
        )
        .layer(middleware::from_fn(auth_middleware))
}

pub fn router_brands() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_brands).post(create_brands))
        .route("/{id}", get(get_brands_by_id).patch(update_brands))
        .layer(middleware::from_fn(auth_middleware))
}

pub fn router_parfume() -> Router<AppState> {
    Router::new()
        .route("/", post(create_parfum))
        .route("/{id}/batch", get(get_all_batch).post(create_batch))
        .route("/{id}/decant", get(get_all_decant).post(create_decant))
        .layer(middleware::from_fn(auth_middleware))
}

pub fn router_batch() -> Router<AppState> {
    Router::new()
        .route("/{id}", patch(update_batch))
        .route("/{id}/bottle", post(create_bottle))
        .layer(middleware::from_fn(auth_middleware))
}

pub async fn create_route(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1/auth", auth_user())
        .nest("/api/v1/users", route_user())
        .nest("/api/v1/brands", router_brands())
        .nest("/api/v1/parfume", router_parfume())
        .nest("/api/v1/batch", router_batch())
        .with_state(state)
}
