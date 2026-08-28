use crate::env::{
    get_cors_origins, get_database_url, get_host, get_max_connections, get_port, init,
};
use crate::route::route::create_route;
use hickory_resolver::TokioResolver;
use hyper::Method;
use hyper::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderValue};
use jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER;
use tower_http::cors::{AllowHeaders, CorsLayer};

mod c_auth;
mod config;
mod dto;
mod env;
mod error;
mod handlers;
mod route;
mod service;
mod state;

use state::AppState;

// middleware simple buat log latency per-request (method + path + status + ms)
async fn log_latency(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = req.method().clone();
    let path = req.uri().path().to_owned();
    let start = std::time::Instant::now();
    let res = next.run(req).await;
    let latency = start.elapsed().as_millis();
    tracing::info!(method=%method, path=%path, status=%res.status().as_u16(), latency_ms=%latency, "request done");
    res
}

#[tokio::main]
async fn main() {
    // logging paling simple: log ke stdout, level dari RUST_LOG (default: info)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    init().expect("Application environment validation failed");

    if let Err(provider) = DEFAULT_PROVIDER.install_default() {
        eprintln!("JWT crypto provider already installed: {:?}", provider);
    }

    let database_url = get_database_url().expect("DATABASE_URL is not available");
    let max_conn = get_max_connections();
    let pool = config::database::connect_db(database_url.as_str(), max_conn)
        .await
        .unwrap();

    let host = get_host();
    let port = get_port();
    let addr = format!("{}:{}", host, port);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(res) => res,
        Err(e) => {
            eprint!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    let cors_origins = get_cors_origins();
    let origins: Vec<HeaderValue> = cors_origins
        .iter()
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
            Method::HEAD,
        ])
        .allow_headers(AllowHeaders::list([CONTENT_TYPE, AUTHORIZATION, ACCEPT]))
        .allow_credentials(true);

    let dns = TokioResolver::builder_tokio()
        .expect("Gagal Membuat DNS")
        .build()
        .unwrap();
    let state = AppState { db: pool, dns: dns };
    let app = create_route(state)
        .await
        .layer(axum::middleware::from_fn(log_latency))
        .layer(cors);

    tracing::info!("listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
