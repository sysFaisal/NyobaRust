use std::io::{self, Write};
use std::sync::Arc;
use std::time::Instant;

use argon2::{
    Algorithm, Argon2, Params, PasswordHasher, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
/* 
=== Result === 12 2 1
Users            : 100
Total time       : 436.47 ms
Average wall time: 4.36 ms
Throughput       : 229.11 password/sec
=== Result === 16 2 1
Users            : 100
Total time       : 598.41 ms
Average wall time: 5.98 ms
Throughput       : 167.11 password/sec
*/
#[tokio::main]
async fn main() {
    println!("=== Argon2id Concurrent Benchmark ===");

    print!("Password: ");
    io::stdout().flush().unwrap();

    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();

    let password = Arc::new(password.trim().to_owned());

    let memory_kib = 12 * 1024;
    let time_cost = 2;
    let parallelism = 1;

    let user_count = 10;

    println!();
    println!("Memory      : {} MiB", memory_kib / 1024);
    println!("Iterations  : {}", time_cost);
    println!("Parallelism : {}", parallelism);
    println!("Users       : {}", user_count);
    println!();

    let params = Params::new(memory_kib, time_cost, parallelism, Some(32))
        .expect("Invalid Argon2 parameters");

    let argon2 = Arc::new(Argon2::new(Algorithm::Argon2id, Version::V0x13, params));

    println!("Starting {} concurrent hashes...", user_count);

    let start = Instant::now();

    let mut handles = Vec::with_capacity(user_count);

    for i in 0..user_count {
        let password = Arc::clone(&password);
        let argon2 = Arc::clone(&argon2);

        let handle = tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);

            let hash = argon2
                .hash_password(password.as_bytes(), &salt)
                .expect("Hashing failed");

            hash.to_string()
        });
        println!("{}", i);
        handles.push(handle);
    }

    // Tunggu semua hashing selesai
    for handle in handles {
        handle.await.expect("Task failed");
    }

    let elapsed = start.elapsed();

    let total_ms = elapsed.as_secs_f64() * 1000.0;
    let throughput = user_count as f64 / elapsed.as_secs_f64();

    println!();
    println!("=== Result ===");
    println!("Users            : {}", user_count);
    println!("Total time       : {:.2} ms", total_ms);
    println!("Average wall time: {:.2} ms", total_ms / user_count as f64);
    println!("Throughput       : {:.2} password/sec", throughput);
}
