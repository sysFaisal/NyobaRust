use std::env;
use std::sync::OnceLock;

pub fn load_env() {
    dotenvy::dotenv().ok();
}

static JWT_SECRET: OnceLock<String> = OnceLock::new();

pub fn init() -> Result<(), String> {
    let key = require("JWT_KEY")?;
    let _ = JWT_SECRET.set(key);
    let _ = require("DATABASE_URL")?;
    Ok(())
}

pub fn get_jwt_key() -> Result<&'static str, String> {
    JWT_SECRET
        .get()
        .map(|s| s.as_str())
        .ok_or_else(|| "JWT_KEY is not initialized; call init() first".to_string())
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

fn optional(key: &str, default: &str) -> String {
    load_env();
    env::var(key).unwrap_or_else(|_| default.to_string())
}

pub fn get_database_url() -> Result<String, String> {
    require("DATABASE_URL")
}

pub fn get_access_token_expiry() -> i64 {
    let val = optional("ACCESS_TOKEN_EXPIRY_MINUTES", "15");
    val.parse().unwrap_or_else(|_| {
        eprintln!("ACCESS_TOKEN_EXPIRY_MINUTES is not a valid number, using default 15");
        15
    })
}

pub fn get_refresh_token_expiry() -> i64 {
    let val = optional("REFRESH_TOKEN_EXPIRY_DAYS", "7");
    val.parse().unwrap_or_else(|_| {
        eprintln!("REFRESH_TOKEN_EXPIRY_DAYS is not a valid number, using default 7");
        7
    })
}

pub fn is_register_enabled() -> bool {
    let val = optional("ENABLE_REGISTER", "true");
    val.parse().unwrap_or_else(|_| {
        eprintln!("ENABLE_REGISTER is not a valid boolean, using default true");
        true
    })
}

pub fn get_host() -> String {
    optional("HOST", "0.0.0.0")
}

pub fn get_port() -> String {
    optional("PORT", "2736")
}

pub fn get_max_connections() -> u32 {
    let val = optional("MAX_CONNECTIONS", "5");
    val.parse().unwrap_or_else(|_| {
        eprintln!("MAX_CONNECTIONS is not a valid number, using default 5");
        5
    })
}

pub fn get_cors_origins() -> Vec<String> {
    let val = optional("CORS_ORIGIN", "http://localhost:3000");
    val.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn is_cookie_secure() -> bool {
    let val = optional("COOKIE_SECURE", "false");
    val.parse().unwrap_or_else(|_| {
        eprintln!("COOKIE_SECURE is not a valid boolean, using default false");
        false
    })
}

pub fn get_log_level() -> String {
    optional("LOG_LEVEL", "info")
}

pub fn get_log_format() -> String {
    optional("LOG_FORMAT", "simple")
}
