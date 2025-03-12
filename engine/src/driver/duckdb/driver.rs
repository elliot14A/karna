use crate::driver::OlapDriver;
use crate::error::*;
use async_trait::async_trait;
use duckdb::DuckdbConnectionManager;
use r2d2::Pool;
use r2d2::PooledConnection;
use serde_json::Value;
use snafu::ResultExt;
use std::collections::HashMap;
use tracing::{debug, info};

use super::config::Config;
use super::utils::{duckdb_row_to_json, sanitize_to_sql_name};

/// DuckDBDriver implements the Driver trait for DuckDB database operations
/// providing a thread-safe interface to execute SQL queries and commands
#[derive(Clone)]
pub struct DuckDBDriver {
    pool: Pool<DuckdbConnectionManager>,
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

        let conn = self.get_connention()?;

        for query in boot_queries {
            debug!("⚙️ Executing boot query: {}", query);
            conn.execute(query, [])
                .context(DuckDBExecutionSnafu { sql: query })?;
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

        let mut stmt = conn.prepare(sql).context(DuckDBPrepareStatementSnafu)?;
        stmt.execute([]).context(DuckDBExecutionSnafu { sql })?;

        info!("✅ Successfully initialized DuckDB driver");
        Ok(())
    }

    fn attach_all_tables(&self) -> Result<()> {
        let db_storage_path = self.config.db_storage_path().display().to_string();
        let db_files = list_db_files(db_storage_path)?;
        info!("🔗 Attaching {} database files", db_files.len());
        for table_name in db_files {
            self.attach_table(table_name)?;
        }
        info!("✅ Successfully attached all database files");
        Ok(())
    }

    pub fn new(config: Config) -> Result<Self> {
        debug!("🔧 Creating new DuckDB driver instance");
        let dsn = config.build_dsn();
        debug!("🔌 Connecting to DuckDB with DSN: {}", dsn);

        let pool_size = config.pool_size().unwrap_or(4);

        let manager = DuckdbConnectionManager::file(dsn).context(DuckDBConnectionSnafu)?;
        let pool = Pool::builder()
            .max_size(pool_size)
            .build(manager)
            .context(DuckDBPoolSnafu)?;
        let driver = DuckDBDriver { pool, config };
        driver.run_boot_queries()?;
        driver.attach_all_tables()?;
        Ok(driver)
    }

    fn get_connention(&self) -> Result<PooledConnection<DuckdbConnectionManager>> {
        self.pool.get().context(DuckDBPoolSnafu)
    }

    fn attach_table(&self, table_name: String) -> Result<()> {
        let conn = self.get_connention()?;
        let sql = format!(
            "attach '{}/{}.db' as {}",
            self.config.db_storage_path().display(),
            table_name,
            table_name
        );
        let mut stmt = conn.prepare(&sql).context(DuckDBPrepareStatementSnafu)?;
        stmt.execute([]).context(DuckDBExecutionSnafu { sql })?;
        Ok(())
    }

    async fn create_table(&self, name: &str, create_sql: &str) -> Result<String> {
        debug!("📝 Creating new table: {}", name);
        let name = sanitize_to_sql_name(name);
        let conn = self.get_connention()?;

        debug!("🔗 Attaching database file");
        let sql = format!(
            "attach '{}/{}.db' as {}",
            self.config.db_storage_path().display(),
            name,
            name
        );
        let mut stmt = conn.prepare(&sql).context(DuckDBPrepareStatementSnafu)?;
        stmt.execute([]).context(DuckDBExecutionSnafu { sql })?;

        debug!("🏗️ Creating table with provided SQL");
        let sql = format!(
            "create or replace table {}.default as ({}\n)",
            name, create_sql
        );
        let mut stmt = conn.prepare(&sql).context(DuckDBPrepareStatementSnafu)?;
        stmt.execute([]).context(DuckDBExecutionSnafu { sql })?;

        debug!("👁️ Creating view for table");
        let sql = self.generate_select_query(name.to_string())?;
        let sql = format!("create or replace view {} as {}", name, sql);
        let mut stmt = conn.prepare(&sql).context(DuckDBPrepareStatementSnafu)?;
        stmt.execute([]).context(DuckDBExecutionSnafu { sql })?;

        info!("✅ Successfully created table and view: {}", name);
        Ok(name)
    }

    pub fn query(&self, sql: &str) -> Result<Vec<HashMap<String, Value>>> {
        debug!("🔍 Executing query: {}", sql);
        let conn = self.get_connention()?;
        let mut stmt = conn.prepare(sql).context(DuckDBPrepareStatementSnafu)?;
        let result = stmt.query([]);
        let mut rows = result.context(DuckDBExecutionSnafu { sql })?;

        let mut rows_data = Vec::new();
        let mut row_count = 0;
        while let Some(row) = rows.next().context(DuckDBNextRowSnafu)? {
            let values = duckdb_row_to_json(row)?;
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
                    .zip(values)
                    .map(|(name, value)| (name.clone(), value))
                    .collect()
            })
            .collect();

        debug!("📊 Query returned {} rows", row_count);
        Ok(result)
    }

    fn generate_select_query(&self, table_name: String) -> Result<String> {
        debug!("🔧 Generating select query for table: {}", table_name);
        let sql = format!(
            r#"
			    select column_name as name
			    from information_schema.columns
			    where table_catalog = '{}' and table_name = 'default';
			"#,
            table_name
        );

        let conn = self.get_connention()?;
        let mut stmt = conn.prepare(&sql).context(DuckDBPrepareStatementSnafu)?;
        let mut rows = stmt.query([]).context(DuckDBExecutionSnafu { sql })?;
        let mut columns = vec![];
        while let Some(row) = rows.next().context(DuckDBNextRowSnafu)? {
            let mut cols = duckdb_row_to_json(row)?;
            columns.push(cols.pop().unwrap().to_string());
        }

        debug!("✨ Generated select query with {} columns", columns.len());
        Ok(format!(
            "select {} from {}.default",
            columns.join(", "),
            table_name
        ))
    }

    fn detach_table(&self, table_name: &str) -> Result<()> {
        debug!("🗑️ Detaching table: {}", table_name);
        let conn = self.get_connention()?;
        let sql = format!("detach {}", table_name);
        let mut stmt = conn.prepare(&sql).context(DuckDBPrepareStatementSnafu)?;
        stmt.execute([]).context(DuckDBExecutionSnafu { sql })?;
        info!("✅ Successfully detached table: {}", table_name);
        Ok(())
    }
}

#[async_trait]
impl OlapDriver for DuckDBDriver {
    fn new(config: Config) -> Result<Self> {
        DuckDBDriver::new(config)
    }

    async fn create_table(&self, table_name: &str, sql: &str) -> Result<String> {
        self.create_table(table_name, sql).await
    }

    async fn query(&self, sql: &str) -> Result<Vec<HashMap<String, Value>>> {
        self.query(sql)
    }

    async fn drop_table(&self, table_name: &str) -> Result<()> {
        // ignore the result of detach_table
        let _ = self.detach_table(table_name);
        // remove the database file
        let path = format!(
            "{}/{}.db",
            self.config.db_storage_path().display(),
            table_name
        );
        tokio::fs::remove_file(&path)
            .await
            .context(FileSystemSnafu { path })?;
        Ok(())
    }
}

/// return a list of files in the database storage path
/// matches all files with the .db extension except main.db file
/// ignores .wal files
fn list_db_files(db_storage_path: String) -> Result<Vec<String>> {
    let mut db_files = vec![];
    let entries = std::fs::read_dir(&db_storage_path).context(FileSystemSnafu {
        path: db_storage_path.clone(),
    })?;

    for entry in entries {
        let entry = entry.context(FileSystemSnafu {
            path: db_storage_path.clone(),
        })?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        if file_name.ends_with(".db") && !file_name.starts_with("main") {
            let wal_file = format!("{}.wal", file_name);
            if !path.exists() || path.is_dir() || path.ends_with(&wal_file) {
                continue;
            }
            let file_name = file_name.replace(".db", "");
            db_files.push(file_name);
        }
    }

    Ok(db_files)
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::*;

    // Helper function to create a test configuration
    fn create_test_config() -> Config {
        let temp_dir = std::env::temp_dir();
        let test_id = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let test_id = format!("test_{test_id}");
        let db_dir = temp_dir.join(&test_id);
        std::fs::create_dir_all(&db_dir).unwrap();

        let db_path = db_dir.join("main.db");
        Config::new(&db_path)
            .unwrap()
            .with_boot_query("CREATE TABLE test (id INTEGER)".to_string())
    }

    #[tokio::test]
    async fn test_driver_initialization() {
        let config = create_test_config();
        let result = DuckDBDriver::new(config);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_boot_queries_execution() {
        // Test that all extensions are properly loaded
        let config = create_test_config();
        let driver = DuckDBDriver::new(config).unwrap();
        let conn = driver.get_connention();
        assert!(conn.is_ok());
        let conn = conn.unwrap();

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
    }

    #[tokio::test]
    async fn test_create_table_and_query() {
        let config = create_test_config();
        let driver = DuckDBDriver::new(config).unwrap();

        // Test simple table creation
        let test_sql = "select * from read_csv('../test-data/seria.csv')";
        let result = driver.create_table("test_table", test_sql).await;
        let table_name = result.unwrap();

        let result = driver.query(format!("select * from {} limit 1", table_name).as_str());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_sanitize_to_sql_name() {
        let name = sanitize_to_sql_name("_testing");
        assert!(name.starts_with("testing"));
        let name = sanitize_to_sql_name("_testing");
        assert!(name.starts_with("testing"));
        let name = sanitize_to_sql_name("_hello!!!world_");
        assert!(name.starts_with("hello_world"));

        let long_name = format!("_{}_", "a".repeat(100));
        assert!(sanitize_to_sql_name(&long_name).len() <= 63);
    }

    #[tokio::test]
    async fn test_detach_table() {
        let config = create_test_config();
        let driver = DuckDBDriver::new(config).unwrap();
        // first attach a table
        let test_sql = "select * from read_csv('../test-data/seria.csv')";
        let table_name = driver.create_table("test_table", test_sql).await.unwrap();

        let result = driver.detach_table(table_name.as_str());
        assert!(result.is_ok());
    }
}
