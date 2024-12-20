pub mod duckdb;

use std::collections::HashMap;

use crate::error::Result;
use async_trait::async_trait;
use duckdb::config::Config;
use serde_json::Value;

/// Trait for OLAP database drivers that support async operations
#[async_trait]
pub trait OlapDriver: Send + Sync + 'static {
    fn new(config: Config) -> Result<Self>
    where
        Self: Sized;

    async fn query(&self, sql: &str) -> Result<Vec<HashMap<String, Value>>>;

    async fn create_table(&self, table_name: &str, sql: &str) -> Result<()>;
}
