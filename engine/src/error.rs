use duckdb::Error as DBError;
use snafu::Snafu;

/// Custom error type for database operations
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    /// Error that occurs when failing to establish a connection to DuckDB
    #[snafu(display("Failed to connect to duckdb connection: {}", source,))]
    ConnectionError { source: DBError },

    /// Error that occurs when failing to close a DuckDB connection after multiple attempts
    #[snafu(display(
        "Failed to close duckdb connection after {retries} retries: {}",
        source,
    ))]
    ClosureError {
        source: DBError, // The underlying DuckDB error
        retries: u8,     // Number of retry attempts made
    },

    /// Error that occurs during SQL query execution
    #[snafu(display("sql execution failed: {}", source,))]
    ExecutionError { source: DBError },
}

/// Result type alias for database operations.
///
/// Generic type parameters:
/// - T: The success type
/// - E: The error type, defaults to our custom Error enum
pub type Result<T, E = Error> = std::result::Result<T, E>;
