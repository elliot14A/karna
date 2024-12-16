mod duckdb;

use crate::error::Result;
use async_trait::async_trait;

/// Trait for OLAP database drivers that support async operations
#[async_trait]
pub trait OlapDriver: Send + Sync + 'static {
    /// Creates a new driver instance from a data source string
    fn new(ds: String) -> Result<Self>
    where
        Self: Sized;

    /// Executes a single SQL statement with no return value
    async fn execute(&self, sql: &str) -> Result<()>;

    /// Executes multiple SQL statements with no return value
    async fn execute_batch(&self, sql: &str) -> Result<()>;

    /// Executes a SQL query and returns the results as JSON
    async fn query(&self, sql: &str) -> Result<serde_json::Value>;
}
