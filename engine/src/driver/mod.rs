mod duckdb;

use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Driver: Send + Sync + 'static {
    fn new(ds: String) -> Result<Self>
    where
        Self: Sized;
    async fn execute(&self, sql: &str) -> Result<()>;
    async fn execute_batch(&self, sql: &str) -> Result<()>;
    async fn query(&self, sql: &str) -> Result<serde_json::Value>;
}
