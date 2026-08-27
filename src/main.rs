use crate::env::{get_database_url, init};
use crate::route::route::create_route;
use hickory_resolver::TokioResolver;
use hyper::header::{HeaderName, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use hyper::Method;
use jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER;
use sqlx::PgPool;
use tower_http::cors::{AllowHeaders, Any, CorsLayer};
use tracing_subscriber::EnvFilter;

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

#[tokio::main]
async fn main() {
    init().expect("Application environment validation failed");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .with_target(false)
        .init();

    if let Err(provider) = DEFAULT_PROVIDER.install_default() {
        eprintln!("JWT crypto provider already installed: {:?}", provider);
    }

    let database_url = get_database_url().expect("DATABASE_URL is not available");
    let pool = config::database::connect_db(database_url.as_str())
        .await
        .unwrap();

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:2736").await {
        Ok(res) => res,
        Err(e) => {
            eprint!("{}", e);
            std::process::exit(1);
        }
    };

// Configure CORS to allow requests from localhost:3000 and 192.168.0.101:3000 with credentials
    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:3000".parse::<HeaderValue>().unwrap(),
            "http://192.168.0.101:3000".parse::<HeaderValue>().unwrap(),
        ])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
            Method::HEAD,
        ])
        .allow_headers(AllowHeaders::list([
            CONTENT_TYPE,
            AUTHORIZATION,
            ACCEPT,
        ]))
        .allow_credentials(true);

    let dns = TokioResolver::builder_tokio()
        .expect("Gagal Membuat DNS")
        .build()
        .unwrap();
    let state = AppState { db: pool, dns: dns };
    let service = create_route(state).await.layer(cors);
    axum::serve(listener, service).await.unwrap();
}
