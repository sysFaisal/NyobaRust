# 🚀 Production Roadmap — NyobaRust Backend

## Status Saat Ini

Kamu sudah punya fondasi yang bagus dengan arsitektur berlapis:

```
Routes -> Handlers -> Services -> SQLx / PostgreSQL
```

| Komponen | Status |
|---|---|
| User CRUD (GET, POST, DELETE) | ✅ Selesai |
| Register + Argon2id hashing | ✅ Selesai |
| Email validation + DNS MX lookup | ✅ Selesai |
| Login + Refresh token generation | ⚠️ Bug: token tidak dikembalikan ke client |
| Auth middleware / guard | ❌ Belum ada |
| JWT Access Token | ❌ Belum ada |
| CORS, Rate Limiting, Logging | ❌ Belum ada |
| Domain (Brands, Parfume, Transaksi) | ❌ Draft saja |
| Database Migrations | ❌ Belum ada |
| Tests | ❌ Belum ada |

---

## Fase 1 — Fix Critical Bugs & Auth Flow ⭐ (Prioritas Tertinggi)

### 1.1 Fix Login: Kembalikan Token ke Client

> ⛔ CRITICAL:
> Saat ini `login_user` (src/handlers/user.rs) menyimpan `token_hash` ke database tapi mengembalikan `data: "".to_string()`. Client tidak pernah menerima token!

Yang perlu dilakukan:
- `svc_login_user()` harus mengembalikan **raw token** (bukan hash) ke handler
- Handler mengembalikan token via **HTTP-only cookie** atau JSON response body
- Pertimbangkan juga mengembalikan user profile di response login

### 1.2 Implementasi JWT Access Token

Refresh token sudah ada, tapi kamu butuh **short-lived access token (JWT)** untuk mengautentikasi setiap request:

```
Login → dapat Refresh Token (7 hari) + Access Token JWT (15 menit)
Request API → kirim Access Token di header `Authorization: Bearer <jwt>`
Token expired → kirim Refresh Token ke /auth/refresh → dapat Access Token baru
```

Dependency yang dibutuhkan:
```toml
jsonwebtoken = "9"
```

File baru yang diperlukan:
- `src/c_auth/jwt.rs` — Fungsi `encode_jwt()`, `decode_jwt()`, dan struct `Claims`
- `src/c_auth/middleware.rs` — Axum extractor untuk validasi JWT dari header

### 1.3 Auth Middleware / Extractor

Buat custom Axum extractor agar route yang butuh autentikasi terlindungi:

```rust
// Contoh penggunaan di route
async fn get_all_user(
    AuthUser(claims): AuthUser,  // ← extractor, otomatis reject kalau belum login
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> { ... }
```

### 1.4 Endpoint Baru untuk Auth

| Method | Route | Fungsi |
|---|---|---|
| `POST` | `/api/v1/auth/refresh` | Tukar refresh token → access token baru |
| `POST` | `/api/v1/auth/logout` | Revoke refresh token (set `revoked_at`) |

---

## Fase 2 — Essential Middleware & Configuration

### 2.1 CORS Middleware

```toml
tower-http = { version = "0.6", features = ["cors", "trace"] }
```

Tanpa CORS, frontend tidak bisa memanggil API kamu dari domain lain.

### 2.2 Request Logging / Tracing

Tambahkan `tracing-subscriber` dan `tower-http::trace::TraceLayer` di `main.rs`:

```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### 2.3 Rate Limiting

Lindungi `/auth/login` dan `/auth/register` dari brute-force:

```toml
tower-governor = "0.4"  # atau implementasi manual dengan in-memory store
```

### 2.4 Environment-based Configuration

Saat ini host/port di-hardcode `0.0.0.0:2736`. Pindahkan ke `.env` atau config struct:

```env
HOST=0.0.0.0
PORT=2736
JWT_SECRET=<random-secret>
JWT_EXPIRY_MINUTES=15
REFRESH_TOKEN_DAYS=7
```

### 2.5 Graceful Shutdown

Tambahkan di `main.rs`:
```rust
let listener = tokio::net::TcpListener::bind(&addr).await?;
axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await?;
```

---

## Fase 3 — Database Migrations

### 3.1 Setup SQLx Migrations

```bash
cargo sqlx migrate add create_users_table
cargo sqlx migrate add create_refresh_tokens_table
cargo sqlx migrate add create_brands_table
cargo sqlx migrate add create_parfumes_table
```

Ini akan membuat folder `migrations/` dengan file SQL berurutan. Ini **wajib** untuk production agar schema database bisa di-track dan di-versioning.

---

## Fase 4 — Domain Implementation (Brands & Parfume)

Berdasarkan `todo otw.txt` dan model draft kamu:

### 4.1 Brands Module

| Method | Route | Fungsi |
|---|---|---|
| `GET` | `/api/v1/brands` | List semua brand |
| `GET` | `/api/v1/brands/{id}` | Detail brand |
| `POST` | `/api/v1/brands` | Buat brand baru (auth required) |
| `PUT` | `/api/v1/brands/{id}` | Update brand (owner only) |
| `DELETE` | `/api/v1/brands/{id}` | Hapus brand (owner only) |

File yang perlu dibuat:
- `src/dto/request/request_brand.rs`
- `src/dto/response/response_brand.rs`
- `src/handlers/brand.rs`
- `src/service/service_brand.rs`

### 4.2 Parfume Module

Sama polanya dengan Brands. Tambahkan juga:
- `batch_parfume` — tracking batch produksi
- `decant` — tracking decant dari batch

### 4.3 Fix Model Syntax Errors

> ⚠️ WARNING:
> File `models/parfume.rs` pakai `boolean` (bukan `bool`) dan `models/user.rs` pakai `Datetime` (bukan `DateTime`).

---

## Fase 5 — User Management (Update)

### 5.1 Implementasi PUT/PATCH User

Saat ini `route.rs` mengarahkan PUT/PATCH ke dummy handler `hello`. Yang perlu dibuat:

- `UpdateUser` DTO (untuk PATCH — partial update)
- `svc_update_user()` service
- `update_user()` handler
- Endpoint: **hanya user sendiri** yang boleh update profile-nya (authorization)

### 5.2 Change Password Endpoint

| Method | Route | Fungsi |
|---|---|---|
| `POST` | `/api/v1/users/change-password` | Ganti password (auth required, verify old password) |

---

## Fase 6 — Production Hardening

### 6.1 Pagination

Semua endpoint `GET` yang mengembalikan list harus support pagination:

```
GET /api/v1/users?page=1&limit=20
GET /api/v1/brands?page=1&limit=20
```

### 6.2 Database Connection Pool Tuning

Saat ini `database.rs` set `max_connections(5)`. Untuk production:

```rust
PgPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(5))
    .idle_timeout(Duration::from_secs(600))
```

### 6.3 Input Sanitization & Security Headers

- Trim & sanitize semua input string
- Tambahkan security headers via middleware (`X-Content-Type-Options`, `X-Frame-Options`, dll)

### 6.4 Health Check Endpoint

```
GET /health → { "status": "ok", "db": "connected" }
```

Penting untuk monitoring & load balancer.

---

## Fase 7 — Testing

### 7.1 Unit Tests

- Test password hashing/verification
- Test email validation
- Test JWT encode/decode
- Test refresh token generation

### 7.2 Integration Tests

- Test full auth flow (register → login → access protected route → refresh → logout)
- Test CRUD user, brand, parfume
- Test error cases (invalid input, unauthorized, not found)

---

## Urutan Pengerjaan yang Disarankan

```
Fase 1: Fix Auth Flow (Login bug, JWT, Middleware)
    ↓
Fase 2: Middleware (CORS, Logging, Config)
    ↓
Fase 3: DB Migrations
    ↓
Fase 4: Domain (Brands, Parfume)
    ↓
Fase 5: User Update (PUT/PATCH, Change Password)
    ↓
Fase 6: Hardening (Pagination, Pool, Health)
    ↓
Fase 7: Testing
```

> ⚠️ IMPORTANT:
> **Mulai dari Fase 1** — tanpa auth flow yang benar, semua fitur lain tidak aman. Setelah auth selesai, lanjut ke middleware (Fase 2) karena CORS dan logging dibutuhkan sejak awal development.

---

## Quick Wins (Bisa Langsung Dikerjakan)

1. ✅ Fix `svc_login_user()` agar return token ke client
2. ✅ Tambah `tracing-subscriber` init di `main.rs` (5 menit)
3. ✅ Pindahkan port ke environment variable (5 menit)
4. ✅ Tambah `/health` endpoint (5 menit)
5. ✅ Fix syntax error di model files
