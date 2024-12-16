use duckdb::Error as DBError;
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Failed to connect to duckdb connection: {}", source,))]
    ConnectionError { source: DBError },

    #[snafu(display(
        "Failed to close duckdb connection after {retries} retries: {}",
        source,
    ))]
    ClosureError { source: DBError, retries: u8 },

    #[snafu(display("sql execution failed: {}", source,))]
    ExecutionError { source: DBError },
}

/// Result type for the engine
pub type Result<T, E = Error> = std::result::Result<T, E>;
