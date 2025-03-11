pub mod datasets;
pub mod middleware;
pub mod query;
use axum::{response::IntoResponse, routing::get, Router};
use engine::driver::{duckdb::driver::DuckDBDriver, sqlx::driver::SqlxDriver};

async fn health_check() -> impl IntoResponse {
    "OK 🏥"
}

pub fn routes() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .nest("/datasets", datasets::routes::<DuckDBDriver, SqlxDriver>())
        .nest("/query", query::router::<DuckDBDriver>())
}
