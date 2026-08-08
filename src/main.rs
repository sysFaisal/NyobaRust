use ::dotenvy::dotenv;
use ::std::env;

use crate::route::route::create_route;

mod config;
mod route;
mod handlers;
mod service;
mod dto;
mod error;

#[tokio::main]
async fn main() {
    dotenv()
    .ok();

    let database_url = env::var("DATABASE_URL").expect("Database Url Not Found");
    let pool = config::database::connect_db(database_url.as_str()).await.unwrap();

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:2736").await {
        Ok(res) => res,
        Err(e) => {
            eprint!("{}", e);
            std::process::exit(1);
        }
    };

    let service = create_route(pool).await;
    axum::serve(listener, service).await.unwrap();
}
