use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use hickory_resolver::TokioResolver;
use mailchecker::is_valid;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};
use std::time::{Duration, Instant};
use tokio::time::timeout;

const MX_CACHE_TTL: Duration = Duration::from_secs(600);
const MX_TIMEOUT: Duration = Duration::from_secs(3);

static MX_CACHE: LazyLock<RwLock<HashMap<String, (Instant, bool)>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);

    let memory_kib = 12 * 1024;
    let time_cost = 2;
    let parallelism = 1;

    let params =
        Params::new(memory_kib, time_cost, parallelism, None).expect("Invalid Argon Params");

    let costum_argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let hash_password = costum_argon2.hash_password(password.as_bytes(), &salt)?;

    Ok(hash_password.to_string())
}

/// Argon2 memakan CPU secara sinkron; jalankan di thread pool agar
/// tokio runtime tidak terblokir (H2).
pub async fn hash_password_async(
    password: &str,
) -> Result<String, argon2::password_hash::Error> {
    let pw = password.to_string();
    tokio::task::spawn_blocking(move || hash_password(&pw)).await.expect(
        "hash_password_async: task spawn_blocking panic atau runtime ditutup",
    )
}

/// Mengembalikan:
/// - `Ok(true)`  : password cocok
/// - `Ok(false)` : password salah (H4: dibedakan dari hash rusak)
/// - `Err(_)`    : hash tidak dapat di-parse (data rusak / config salah)
pub fn verify_password(
    password: &str,
    hashed_password: &str,
) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hashed_password)?;

    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(e),
    }
}

/// Sama dengan `verify_password` namun dijalankan di thread pool (H2).
pub async fn verify_password_async(
    password: &str,
    hashed_password: &str,
) -> Result<bool, argon2::password_hash::Error> {
    let pw = password.to_string();
    let hash = hashed_password.to_string();
    tokio::task::spawn_blocking(move || verify_password(&pw, &hash)).await.expect(
        "verify_password_async: task spawn_blocking panic atau runtime ditutup",
    )
}

pub async fn validate_email(dns: &TokioResolver, email: &str) -> bool {
    if !is_valid(email) {
        return false;
    }

    let Some((_, domain)) = email.rsplit_once('@') else {
        return false;
    };

    if let Some(cached) = MX_CACHE
        .read()
        .ok()
        .and_then(|m| m.get(domain).filter(|(t, _)| t.elapsed() < MX_CACHE_TTL).map(|(_, v)| *v))
    {
        return cached;
    }

    let result = match timeout(MX_TIMEOUT, dns.mx_lookup(domain)).await {
        Ok(Ok(lookup)) => !lookup.answers().is_empty(),
        Ok(Err(_)) | Err(_) => false,
    };

    if let Ok(mut m) = MX_CACHE.write() {
        m.insert(domain.to_string(), (Instant::now(), result));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_password_distinguishes_wrong_password_from_broken_hash() {
        let good = hash_password("secret-123").unwrap();
        assert!(matches!(verify_password("secret-123", &good), Ok(true)));
        assert!(matches!(
            verify_password("wrong-password", &good),
            Ok(false)
        ));

        // Hash yang rusak/format salah -> Err (bukan Ok(false))
        assert!(verify_password("anything", "not-a-valid-hash").is_err());
    }
}

