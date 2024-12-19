use crate::error::*;
use duckdb::{Connection, Transaction};
use snafu::ResultExt;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;

// use crate::driver::OlapDriver;

use super::config::Config;
use super::utils::duckdb_row_to_json;

/// DuckDBDriver implements the Driver trait for DuckDB database operations
/// providing a thread-safe interface to execute SQL queries and commands
/// TODO: maybe implement bb8::ManageConnection for connection pooling
/// Have separate connections for read and write operations
pub struct DuckDBDriver {
    /// Thread-safe connection to DuckDB
    conn: Arc<Mutex<Connection>>,
    config: Config,
}

impl DuckDBDriver {
    async fn run_boot_queries(&self) -> Result<()> {
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

        let conn = self.conn.lock().await;

        for query in boot_queries {
            conn.execute(query, [])
                .context(ExecutionSnafu { sql: query })?;
        }

        // Forces DuckDB to create catalog entries for the information schema up front (they are normally created lazily).
        // Obviously, copied directly from rill codebase
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

        Ok(())
    }

    pub async fn new(config: Config) -> Result<Self> {
        let dsn = config.build_dsn();
        let conn = Connection::open(dsn).context(ConnectionSnafu)?;
        let driver = DuckDBDriver {
            conn: Arc::new(Mutex::new(conn)),
            config,
        };
        driver.run_boot_queries().await?;
        Ok(driver)
    }

    async fn create_table(&self, name: &str, create_sql: &str) -> Result<()> {
        let name = sanitize_to_sql_name(name);
        let path = self.config.db_storage_path().join(format!("{}.db", name));
        // create new duckdb file
        fs::File::create(&path)
            .await
            .context(FileSystemSnafu { path: name.clone() })?;

        let mut conn = self.conn.lock().await;

        let transaction = conn.transaction().context(TransactionSnafu)?;

        // attach the new duckdb file to main.db
        let sql = format!("attach {} as {}", format!("'./{}.db'", name), name);
        let mut stmt = transaction.prepare(&sql).context(PrepareStatementSnafu)?;
        stmt.execute([]).context(ExecutionSnafu { sql })?;

        // load file in to the new db file
        let sql = format!(
            "create or replace table {}.default as ({}\n)",
            name, create_sql
        );
        let mut stmt = transaction.prepare(&sql).context(PrepareStatementSnafu)?;
        stmt.execute([]).context(ExecutionSnafu { sql })?;

        transaction.commit().context(TransactionSnafu)?;

        // create new transaction to create view
        let transaction = conn.transaction().context(TransactionSnafu)?;
        let sql = Self::generate_select_query(&transaction, name.to_string()).await?;
        let sql = format!("create or replace view {} as {}", name, sql);
        let mut stmt = transaction.prepare(&sql).context(PrepareStatementSnafu)?;
        stmt.execute([]).context(ExecutionSnafu { sql })?;
        transaction.commit().context(TransactionSnafu)?;

        Ok(())
    }

    async fn generate_select_query(
        transaction: &Transaction<'_>,
        table_name: String,
    ) -> Result<String> {
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

        return Ok(format!(
            "select {} from {}.default",
            columns.join(", "),
            table_name
        ));
    }
}

fn sanitize_to_sql_name(filename: &str) -> String {
    const MAX_LENGTH: usize = 63; // Common SQL identifier length limit

    // Sanitize the filename:
    // 1. Replace non-alphanumeric chars with underscore
    // 2. Remove consecutive underscores
    // 3. Remove leading/trailing underscores
    let sanitized: String = filename
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_");

    // If the sanitized string starts with a number, prepend 'n'
    let valid_start = if sanitized
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        format!("n{}", sanitized)
    } else {
        sanitized
    };

    // Truncate if necessary, ensuring we don't cut in the middle of an underscore
    if valid_start.len() > MAX_LENGTH {
        let truncated = &valid_start[..MAX_LENGTH];
        match truncated.rfind('_') {
            Some(pos) if pos > 0 => valid_start[..pos].to_string(),
            _ => truncated.to_string(),
        }
    } else {
        valid_start
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
        let result = DuckDBDriver::new(config).await;
        assert!(result.is_ok());
        clean_up(db).await.unwrap();
    }

    #[tokio::test]
    async fn test_boot_queries_execution() {
        // Test that all extensions are properly loaded
        let db = "test2.db".to_string();
        clean_up(db.clone()).await.unwrap();
        let config = create_test_config(db.clone());
        let driver = DuckDBDriver::new(config).await.unwrap();
        let conn = driver.conn.lock().await;

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
    async fn test_create_table() {
        let db = "test3.db".to_string();
        clean_up("test_table.db".to_string()).await.unwrap();
        clean_up(db.clone()).await.unwrap();
        let config = create_test_config(db.clone());
        let driver = DuckDBDriver::new(config).await.unwrap();

        // Test simple table creation
        let test_sql = "select * from read_csv('../test-data/deliveries.csv')";
        let result = driver.create_table("test_table", test_sql).await;
        assert!(result.is_ok());

        // Verify table creation and data
        let conn = driver.conn.lock().await;
        let mut stmt = conn
            .prepare("SELECT * FROM test_table.default LIMIT 1")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        let res = rows.next().unwrap();
        assert!(res.is_some());
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
