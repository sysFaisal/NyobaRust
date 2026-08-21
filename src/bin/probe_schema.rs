use sqlx::Row;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL")?).await?;

    let rows = sqlx::query(
        "SELECT column_name, is_nullable, data_type \
         FROM information_schema.columns \
         WHERE table_name = 'users' ORDER BY ordinal_position",
    )
    .fetch_all(&pool)
    .await?;

    println!("column | nullable | type");
    for r in rows {
        let name: String = r.get("column_name");
        let nullable: String = r.get("is_nullable");
        let dtype: String = r.get("data_type");
        println!("{name} | {nullable} | {dtype}");
    }

    Ok(())
}
