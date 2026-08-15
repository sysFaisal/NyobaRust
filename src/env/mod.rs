use std::env;

pub fn load_env() {
    dotenvy::dotenv().ok();
}

fn require(key: &str) -> Result<String, String> {
    load_env();

    let value = env::var(key).map_err(|_| {
        format!(
            "{} is required but missing. Please set it in the .env file or process environment.",
            key
        )
    })?;

    if value.trim().is_empty() {
        return Err(format!("{} cannot be empty.", key));
    }

    Ok(value)
}

pub fn init() -> Result<(), String> {
    let _ = require("JWT_KEY")?;
    let _ = require("DATABASE_URL")?;
    Ok(())
}

pub fn get_jwt_key() -> Result<String, String> {
    require("JWT_KEY")
}

pub fn get_database_url() -> Result<String, String> {
    require("DATABASE_URL")
}
