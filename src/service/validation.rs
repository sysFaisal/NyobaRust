use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use hickory_resolver::TokioResolver;
use mailchecker::is_valid;

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

pub fn verify_password(
    password: &str,
    hashed_password: &str,
) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hashed_password)?;

    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => return Ok(true),
        Err(_) => return Ok(false),
    };
}

pub async fn validate_email(dns: &TokioResolver, email: &str) -> bool {
    if !is_valid(email) {
        return false;
    }

    let Some((_, domain)) = email.rsplit_once('@') else {
        return false;
    };

    match dns.mx_lookup(domain).await {
        Ok(lookup) => !lookup.answers().is_empty(),
        Err(_) => false,
    }
}



