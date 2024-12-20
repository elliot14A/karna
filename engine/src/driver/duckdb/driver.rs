use crate::error::*;
use async_trait::async_trait;
use duckdb::Connection;
use serde_json::Value;
use snafu::ResultExt;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::{Mutex, MutexGuard};
use tokio::fs;
use tracing::{debug, info};

use crate::driver::OlapDriver;

use super::config::Config;
use super::utils::{duckdb_row_to_json, sanitize_to_sql_name};

/// DuckDBDriver implements the Driver trait for DuckDB database operations
/// providing a thread-safe interface to execute SQL queries and commands
pub struct DuckDBDriver {
    conn: Arc<Mutex<Connection>>,
    config: Config,
}

impl DuckDBDriver {
    fn run_boot_queries(&self) -> Result<()> {
        debug!("🚀 Initializing DuckDB extensions and boot queries");

        let mut boot_queries = vec![
            "install 'json'",
            "load 'json'",
            "install 'icu'",
            "load 'icu'",
            "install 'parquet'",
            "load 'parquet'",
            "install 'sqlite'",
            "load 'sqlite'",
            "install 'httpfs'",
            "load 'httpfs'",
        ];

        boot_queries.extend(self.config.boot_queries().iter().map(String::as_str));

        let conn = self.conn.lock().unwrap();

        for query in boot_queries {
            debug!("⚙️ Executing boot query: {}", query);
            conn.execute(query, [])
                .context(ExecutionSnafu { sql: query })?;
        }

        debug!("📊 Initializing information schema");
        let sql = r#"
	          select
			    coalesce(t.table_catalog, current_database()) as "database",
			    t.table_schema as "schema",
			    t.table_name as "name",
			    t.table_type as "type", 
			    array_agg(c.column_name order by c.ordinal_position) as "column_names",
			    array_agg(c.data_type order by c.ordinal_position) as "column_types",
			    array_agg(c.is_nullable = 'YES' order by c.ordinal_position) as "column_nullable"
		        from information_schema.tables t
		        join information_schema.columns c on t.table_schema = c.table_schema and t.table_name = c.table_name
		        group by 1, 2, 3, 4
              order by 1, 2, 3, 4
        "#;

        let mut stmt = conn.prepare(sql).context(PrepareStatementSnafu)?;
        stmt.execute([]).context(ExecutionSnafu { sql })?;

        info!("✅ Successfully initialized DuckDB driver");
        Ok(())
    }

    pub fn new(config: Config) -> Result<Self> {
        debug!("🔧 Creating new DuckDB driver instance");
        let dsn = config.build_dsn();
        debug!("🔌 Connecting to DuckDB with DSN: {}", dsn);

        let conn = Connection::open(dsn).context(ConnectionSnafu)?;
        let driver = DuckDBDriver {
            conn: Arc::new(Mutex::new(conn)),
            config,
        };
        driver.run_boot_queries()?;
        Ok(driver)
    }

    async fn create_table(&self, name: &str, create_sql: &str) -> Result<()> {
        debug!("📝 Creating new table: {}", name);
        let name = sanitize_to_sql_name(name);
        let path = self.config.db_storage_path().join(format!("{}.db", name));

        debug!("💾 Creating database file at: {:?}", path);
        fs::File::create(&path)
            .await
            .context(FileSystemSnafu { path: name.clone() })?;

        let conn = self.conn.lock().unwrap();

        debug!("🔗 Attaching database file");
        let sql = format!("attach {} as {}", format!("'./{}.db'", name), name);
        let mut stmt = conn.prepare(&sql).context(PrepareStatementSnafu)?;
        stmt.execute([]).context(ExecutionSnafu { sql })?;

        debug!("🏗️ Creating table with provided SQL");
        let sql = format!(
            "create or replace table {}.default as ({}\n)",
            name, create_sql
        );
        let mut stmt = conn.prepare(&sql).context(PrepareStatementSnafu)?;
        stmt.execute([]).context(ExecutionSnafu { sql })?;

        debug!("👁️ Creating view for table");
        let sql = Self::generate_select_query(&conn, name.to_string())?;
        let sql = format!("create or replace view {} as {}", name, sql);
        let mut stmt = conn.prepare(&sql).context(PrepareStatementSnafu)?;
        stmt.execute([]).context(ExecutionSnafu { sql })?;

        info!("✅ Successfully created table and view: {}", name);
        Ok(())
    }

    pub fn query(&self, sql: &str) -> Result<Vec<HashMap<String, Value>>> {
        debug!("🔍 Executing query: {}", sql);
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(sql).context(PrepareStatementSnafu)?;
        let mut rows = stmt.query([]).context(ExecutionSnafu { sql })?;

        let mut rows_data = Vec::new();
        let mut row_count = 0;
        while let Some(row) = rows.next().context(NextRowSnafu)? {
            let values = duckdb_row_to_json(&row)?;
            rows_data.push(values);
            row_count += 1;
        }

        let schema = stmt.schema();
        let column_names: Vec<String> = schema
            .fields()
            .iter()
            .map(|field| field.name().to_string())
            .collect();

        let result = rows_data
            .into_iter()
            .map(|values| {
                column_names
                    .iter()
                    .zip(values.into_iter())
                    .map(|(name, value)| (name.clone(), value))
                    .collect()
            })
            .collect();

        debug!("📊 Query returned {} rows", row_count);
        Ok(result)
    }

    fn generate_select_query(
        transaction: &MutexGuard<'_, Connection>,
        table_name: String,
    ) -> Result<String> {
        debug!("🔧 Generating select query for table: {}", table_name);
        let sql = format!(
            r#"
			select column_name as name
			from information_schema.columns
			where table_catalog = '{}' and table_name = 'default'
			order by name asc
        "#,
            table_name
        );
        let mut stmt = transaction.prepare(&sql).context(PrepareStatementSnafu)?;
        let mut rows = stmt.query([]).context(ExecutionSnafu { sql })?;
        let mut columns = vec![];
        while let Some(row) = rows.next().context(NextRowSnafu)? {
            let mut cols = duckdb_row_to_json(row)?;
            columns.push(cols.pop().unwrap().to_string());
        }

        debug!("✨ Generated select query with {} columns", columns.len());
        return Ok(format!(
            "select {} from {}.default",
            columns.join(", "),
            table_name
        ));
    }
}

#[async_trait]
impl OlapDriver for DuckDBDriver {
    fn new(config: Config) -> Result<Self> {
        DuckDBDriver::new(config)
    }

    async fn create_table(&self, table_name: &str, sql: &str) -> Result<()> {
        self.create_table(table_name, sql).await
    }

    async fn query(&self, sql: &str) -> Result<Vec<HashMap<String, Value>>> {
        self.query(sql)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a test configuration
    fn create_test_config(name: String) -> Config {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(&name);
        Config::new(&db_path)
            .unwrap()
            .with_boot_query("CREATE TABLE test (id INTEGER)".to_string())
    }

    async fn clean_up(name: String) -> std::io::Result<()> {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(&name);
        let db_dir = temp_dir.join(&name);

        if db_path.exists() {
            fs::remove_file(&db_path).await?;
        }

        if db_dir.exists() {
            fs::remove_dir_all(&db_dir).await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_driver_initialization() {
        let db = "test1.db".to_string();
        clean_up(db.clone()).await.unwrap();
        let config = create_test_config(db.clone());
        let result = DuckDBDriver::new(config);
        assert!(result.is_ok());
        clean_up(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_boot_queries_execution() {
        // Test that all extensions are properly loaded
        let db = "test2.db".to_string();
        clean_up(db.clone()).await.unwrap();
        let config = create_test_config(db.clone());
        let driver = DuckDBDriver::new(config).unwrap();
        let conn = driver.conn.lock().unwrap();

        // Test JSON extension
        let result = conn.execute("SELECT json_structure('[1,2,3]')", []);
        assert!(result.is_ok());

        // Test ICU extension
        let result = conn.execute("SELECT lower('HELLO')", []);
        assert!(result.is_ok());

        // Test Parquet extension
        let result = conn.execute("SET enable_progress_bar=false", []);
        assert!(result.is_ok());

        // Test SQLite extension
        let result = conn.execute("SET sqlite_all_varchar=false", []);
        assert!(result.is_ok());
        clean_up(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_create_table_and_query() {
        let db = "test3.db".to_string();
        clean_up("test_table.db".to_string()).await.unwrap();
        clean_up(db.clone()).await.unwrap();
        let config = create_test_config(db.clone());
        let driver = DuckDBDriver::new(config).unwrap();

        // Test simple table creation
        let test_sql = "select * from read_csv('../test-data/deliveries.csv')";
        let result = driver.create_table("test_table", test_sql).await;
        assert!(result.is_ok());

        let result = driver.query("select * from test_table limit 1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
        clean_up(db).await.unwrap();
        clean_up("test_table.db".to_string()).await.unwrap();
    }

    #[test]
    fn test_sanitize_to_sql_name() {
        assert_eq!(sanitize_to_sql_name("hello world"), "hello_world");
        assert_eq!(sanitize_to_sql_name("hello!!!world"), "hello_world");
        assert_eq!(sanitize_to_sql_name("123table"), "n123table");
        assert_eq!(sanitize_to_sql_name("$#@!table^&*"), "table");
        assert_eq!(sanitize_to_sql_name("_hello_world_"), "hello_world");
        assert_eq!(sanitize_to_sql_name("HelloWorld"), "HelloWorld");

        // Test maximum length
        let long_name = "a".repeat(100);
        assert!(sanitize_to_sql_name(&long_name).len() <= 63);
    }
}
