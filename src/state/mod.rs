use hickory_resolver::TokioResolver;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub dns: TokioResolver,
}
