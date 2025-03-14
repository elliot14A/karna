use duckdb::Error as DBError;
use snafu::Snafu;

/// Custom error type for database operations
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Failed to get duckdb connection from pool: {source}"))]
    DuckDBConnection { source: DBError },

    #[snafu(display("Pool error: {source}"))]
    DuckDBPool { source: r2d2::Error },

    #[snafu(display("Failed to prepare duckdb statement: {source}"))]
    DuckDBPrepareStatement { source: DBError },

    #[snafu(display("Failed to convert DuckDB value: {message}"))]
    DuckDBValueConversion { message: String },

    #[snafu(display("Failed to execute duckdb query '{sql}': {source}"))]
    DuckDBExecution { source: DBError, sql: String },

    #[snafu(display("Transaction error: {source}"))]
    DuckDBTransaction { source: DBError },

    #[snafu(display("File system error at '{path}': {source}"))]
    FileSystem {
        source: std::io::Error,
        path: String,
    },

    #[snafu(display("Karna enginre does not support the format: {format}"))]
    InvalidFormat { format: String },

    #[snafu(display("Failed to get next row: {source}"))]
    DuckDBNextRow { source: DBError },

    #[snafu(display("Invalid configuration: {message}"))]
    Config { message: String },

    #[snafu(display("Failed to connect to SQLx database: {source}"))]
    SqlxConnection { source: sqlx::Error },

    #[snafu(display("Failed to execute SQLx query: {sql}: {source}"))]
    SqlxExecute { sql: String, source: sqlx::Error },

    #[snafu(display("Failed to query using SQLx: {sql}: {source}"))]
    SqlxQuery { sql: String, source: sqlx::Error },

    #[snafu(display("Migration directory not found: {path}"))]
    MigrationDirNotFound { path: String },

    #[snafu(display("Failed to run SQLx migrations: {source}"))]
    SqlxMigration { source: sqlx::migrate::MigrateError },

    #[snafu(display("Failed to convert string to datetime: {value}: {source}"))]
    DateTimeParse {
        source: chrono::ParseError,
        value: String,
    },
}

/// Result type alias for database operations.
///
/// Generic type parameters:
/// - T: The success type
/// - E: The error type, defaults to our custom Error enum
pub type Result<T, E = Error> = std::result::Result<T, E>;
