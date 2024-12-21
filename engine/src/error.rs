use duckdb::Error as DBError;
use snafu::Snafu;

/// Custom error type for database operations
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Failed to connect to duckdb: {source}"))]
    DuckDBConnection { source: DBError },

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

    #[snafu(display("Failed to connect to libsql: {source}"))]
    LibSQLConnection { source: libsql::Error },

    #[snafu(display("Failed to execute libsql query '{sql}': {source}"))]
    LibSQLExecute { source: libsql::Error, sql: String },

    #[snafu(display("Failed to get next row from libsql: {source}"))]
    LibSQLNextRow { source: libsql::Error },

    #[snafu(display("Failed to convert libsql value: {message}"))]
    LibSQLConverstion { message: String },

    #[snafu(display("Failed to prepare libsql statement '{sql}' : {source}"))]
    LibSQLPrepareStatement { source: libsql::Error, sql: String },
}

/// Result type alias for database operations.
///
/// Generic type parameters:
/// - T: The success type
/// - E: The error type, defaults to our custom Error enum
pub type Result<T, E = Error> = std::result::Result<T, E>;
