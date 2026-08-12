use crate::route::route::create_route;
use ::dotenvy::dotenv;
use ::std::env;
use hickory_resolver::TokioResolver;
use sqlx::PgPool;

mod config;
mod dto;
mod error;
mod handlers;
mod route;
mod service;
mod state;

use state::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("Database Url Not Found");
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
    let dns = TokioResolver::builder_tokio()
        .expect("Gagal Membuat DNS")
        .build()
        .unwrap();
    let state = AppState { db: pool, dns: dns };
    let service = create_route(state).await;
    axum::serve(listener, service).await.unwrap();
}
