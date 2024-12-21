pub mod duckdb;
pub mod libsql;

use std::collections::HashMap;

use crate::{error::Result, models};
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

#[async_trait]
pub trait DatasetStoreDriver: Send + Sync + 'static {
    async fn create(&self, dataset: &models::CreateDataset) -> Result<models::Dataset>;

    async fn details(&self, id: &uuid::Uuid) -> Result<models::Dataset>;

    async fn update(&self, dataset: &models::UpdateDataset) -> Result<models::Dataset>;

    async fn delete(&self, id: &uuid::Uuid) -> Result<()>;

    async fn list(&self) -> Result<Vec<models::Dataset>>;
}
