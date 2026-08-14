use std::alloc::System;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::format_description::well_known::iso8601::FormattedComponents::Date;
use uuid::Uuid;

use crate::error::error::AppError;
use std::env::var;

#[derive(Serialize, Deserialize)]
pub struct RefreshToken {
    pub token: String,
    pub token_hash: String,
    pub expire_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct AccesClaims {
    pub sub: String, //Uuid
    pub iat: i64,    //created_token
    pub exp: i64,    //expire_token
}

pub fn generate_access_token(user_id: Uuid) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = AccesClaims {
        sub: user_id.to_string(),
        iat: now.timestamp(),
        exp: (now + Duration::minutes(5)).timestamp(), //5 menit
    };

    let secret = env::var("JWT_KEY").expect(return Err(AppError::InternalServerError(None)));
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect(return Err(AppError::InternalServerError(None)));
    Ok(token)
}

pub fn hash_token_sha256(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

pub fn generate_refresh_token() -> RefreshToken {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let token = hex::encode(bytes);
    let token_hash = hex::encode(hash_token_sha256(&bytes));
    let expire_at = Utc::now() + Duration::days(7);

    RefreshToken {
        token,
        token_hash,
        expire_at,
    }
}

pub fn token_to_hash_token(token: &String) -> String {
    let hash = hex::encode(Sha256::digest(token.as_bytes()));
    return hash;
}
