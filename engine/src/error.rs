use duckdb::Error as DBError;
use snafu::Snafu;

/// Custom error type for database operations
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Failed to connect to database: {source}"))]
    Connection { source: DBError },

    #[snafu(display("Failed to prepare statement: {source}"))]
    PrepareStatement { source: DBError },

    #[snafu(display("Failed to convert DuckDB value: {message}"))]
    DuckDBValueConversion { message: String },

    #[snafu(display("Failed to execute query '{sql}': {source}"))]
    Execution { source: DBError, sql: String },

    #[snafu(display("Transaction error: {source}"))]
    Transaction { source: DBError },

    #[snafu(display("File system error at '{path}': {source}"))]
    FileSystem {
        source: std::io::Error,
        path: String,
    },

    #[snafu(display("Karna enginre does not support the format: {format}"))]
    InvalidFormat { format: String },

    #[snafu(display("Failed to get next row: {source}"))]
    NextRow { source: DBError },

    #[snafu(display("Invalid configuration: {message}"))]
    Config { message: String },
}

/// Result type alias for database operations.
///
/// Generic type parameters:
/// - T: The success type
/// - E: The error type, defaults to our custom Error enum
pub type Result<T, E = Error> = std::result::Result<T, E>;
