use crate::{
    driver::duckdb::utils::duckdb_value_to_json,
    error::{ConnectionSnafu, ExecutionSnafu, Result},
};
use async_trait::async_trait;
use duckdb::{params, Connection};
use snafu::ResultExt;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::driver::Driver;

/// DuckDBDriver implements the Driver trait for DuckDB database operations
/// providing a thread-safe interface to execute SQL queries and commands
pub struct DuckDBDriver {
    /// Thread-safe connection to DuckDB
    conn: Arc<Mutex<Connection>>,
}

impl DuckDBDriver {
    /// Creates a new DuckDBDriver instance
    ///
    /// # Arguments
    /// * `dsn` - Data Source Name string for connecting to DuckDB
    ///
    /// # Returns
    /// * `Result<Self>` - New DuckDBDriver instance wrapped in Result
    pub fn new(dsn: String) -> Result<Self> {
        let conn = Connection::open(dsn).context(ConnectionSnafu)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

#[async_trait]
impl Driver for DuckDBDriver {
    /// Implementation of Driver::new for DuckDBDriver
    fn new(dsn: String) -> Result<Self> {
        Self::new(dsn)
    }

    /// Executes a single SQL statement that doesn't return results
    ///
    /// # Arguments
    /// * `sql` - SQL statement to execute
    async fn execute(&self, sql: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(sql, []).context(ConnectionSnafu)?;
        Ok(())
    }

    /// Executes multiple SQL statements in batch mode
    ///
    /// # Arguments
    /// * `sql` - SQL statements to execute
    async fn execute_batch(&self, sql: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute_batch(sql).context(ConnectionSnafu)?;
        Ok(())
    }

    /// Executes a SQL query and returns the results as JSON
    ///
    /// # Arguments
    /// * `sql` - SQL query to execute
    ///
    /// # Returns
    /// * `Result<serde_json::Value>` - Query results as JSON array
    async fn query(&self, sql: &str) -> Result<serde_json::Value> {
        // Acquire connection lock
        let conn = self.conn.lock().await;

        // Prepare and execute query
        let mut stmt = conn.prepare(sql).context(ExecutionSnafu)?;
        let columns = stmt.column_names();

        let mut rows = stmt.query(params![]).context(ExecutionSnafu)?;
        let mut json_array = Vec::new();

        // Iterate through results and build JSON array
        while let Some(row) = rows.next().context(ExecutionSnafu)? {
            let mut row_obj = serde_json::Map::new();

            // Convert each column value to JSON
            for (i, column) in columns.iter().enumerate() {
                let value = row.get(i).context(ExecutionSnafu)?;
                let json_value = duckdb_value_to_json(value)?;
                row_obj.insert(column.to_string(), json_value);
            }

            json_array.push(serde_json::Value::Object(row_obj));
        }

        Ok(serde_json::Value::Array(json_array))
    }
}
