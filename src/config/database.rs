use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub async fn connect_db(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
}
