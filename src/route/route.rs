use crate::handlers::batch::{create_batch, delete_batch, get_all_batch, update_batch};
use crate::handlers::bottle::{
    create_bottle, delete_bottle, get_all_bottle, get_bottle, update_bottle,
};
use crate::handlers::brand::{
    create_brands, delete_brands, get_all_brands, get_brands_by_id, update_brands,
};
use crate::handlers::decant::{create_decant, delete_decant, get_all_decant, update_decant};
use crate::handlers::order::{create_order, get_all_order, update_order};
use crate::handlers::parfume::{
    create_parfum, delete_parfume, get_all_parfume, get_all_parfume_uni, get_parfume_by_id,
    get_parfume_history, get_parfume_ranking, update_parfume,
};
use crate::handlers::revenue::{get_daily_revenue, get_revenue_history, get_revenue_summary};
use crate::handlers::user::{
    create_user, delete_data_user, get_all_user, get_user_by_id, login_user, logout_user,
    refresh_token, update_user,
};
use crate::service::brands_svc::svc_get_all_brands;
use crate::service::user_svc::auth_middleware;
use crate::env::is_register_enabled;
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
    let mut router = Router::new()
        .route("/login", post(login_user))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout_user));

    if is_register_enabled() {
        router = router.route("/register", post(create_user));
    }

    router
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
        .route(
            "/{id}",
            get(get_brands_by_id)
                .patch(update_brands)
                .delete(delete_brands),
        )
        .route("/{id}/parfume", get(get_all_parfume))
        .layer(middleware::from_fn(auth_middleware))
}

pub fn router_parfume() -> Router<AppState> {
    Router::new()
        .route("/", post(create_parfum).get(get_all_parfume_uni))
        .route(
            "/{id}",
            get(get_parfume_by_id)
                .patch(update_parfume)
                .delete(delete_parfume),
        )
        .route("/{id}/batch", get(get_all_batch).post(create_batch))
        .route("/{id}/decant", get(get_all_decant).post(create_decant))
        .route("/{id}/history", get(get_parfume_history))
        .route("/ranking", get(get_parfume_ranking))
        .layer(middleware::from_fn(auth_middleware))
}

pub fn router_batch() -> Router<AppState> {
    Router::new()
        .route("/{id}", patch(update_batch).delete(delete_batch))
        .route("/{id}/bottle", post(create_bottle).get(get_all_bottle))
        .layer(middleware::from_fn(auth_middleware))
}

pub fn router_decant() -> Router<AppState> {
    Router::new()
        .route("/{id}", patch(update_decant).delete(delete_decant))
        .layer(middleware::from_fn(auth_middleware))
}

pub fn router_bottle() -> Router<AppState> {
    Router::new()
        .route(
            "/{id}",
            get(get_bottle).patch(update_bottle).delete(delete_bottle),
        )
        .layer(middleware::from_fn(auth_middleware))
}

pub fn router_order() -> Router<AppState> {
    Router::new()
        .route("/", post(create_order).get(get_all_order))
        .route("/{id}", patch(update_order))
        .layer(middleware::from_fn(auth_middleware))
}

pub fn router_revenue() -> Router<AppState> {
    Router::new()
        .route("/summary", get(get_revenue_summary))
        .route("/daily", get(get_daily_revenue))
        .route("/history", get(get_revenue_history))
        .layer(middleware::from_fn(auth_middleware))
}

pub async fn create_route(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1/auth", auth_user())
        .nest("/api/v1/users", route_user())
        .nest("/api/v1/brands", router_brands())
        .nest("/api/v1/parfume", router_parfume())
        .nest("/api/v1/batch", router_batch())
        .nest("/api/v1/bottle", router_bottle())
        .nest("/api/v1/decant", router_decant())
        .nest("/api/v1/order", router_order())
        .nest("/api/v1/revenue", router_revenue())
        .with_state(state)
}
