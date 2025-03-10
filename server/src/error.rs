use axum::{
    extract::multipart::MultipartError,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Failed to read multipart: {source}"))]
    MultiPart { source: MultipartError },

    #[snafu(display("Internal error: {}", message))]
    Internal { message: String },

    #[snafu(display("Failed to read file: {message}"))]
    BadReq { message: String },

    #[snafu(display("Failed operation on file message: {message}, error: {source}"))]
    FileError {
        source: std::io::Error,
        message: String,
    },

    #[snafu(display("Not found: {message}"))]
    NotFound { message: String },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Serialize)]
struct ErrorMessage {
    message: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound { message } => (StatusCode::NOT_FOUND, message),
            Self::MultiPart { source } => (
                StatusCode::BAD_REQUEST,
                format!("Failed to read multipart: {}", source.to_string()),
            ),
            Self::Internal { message } => (StatusCode::INTERNAL_SERVER_ERROR, message),
            Self::BadReq { message } => (StatusCode::BAD_REQUEST, message),
            Self::FileError { message, .. } => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };

        let error_message = ErrorMessage { message };
        (status, axum::Json(error_message)).into_response()
    }
}

impl From<engine::error::Error> for Error {
    fn from(value: engine::error::Error) -> Self {
        match value {
            engine::error::Error::DuckDBConnection { source } => Self::Internal {
                message: format!("failed to connect to duckdb, error: {source}"),
            },

            engine::error::Error::DuckDBPrepareStatement { source } => Self::BadReq {
                message: format!("prepare statement error: {source}"),
            },

            engine::error::Error::DuckDBValueConversion { message } => Self::Internal {
                message: format!("failed to convert value, error: {message}"),
            },

            engine::error::Error::DuckDBExecution { source, sql } => Self::Internal {
                message: format!("failed to execute sql '{sql}', error: {source}"),
            },

            engine::error::Error::DuckDBTransaction { source } => Self::Internal {
                message: format!("failed to execute transaction, error: {source}"),
            },

            engine::error::Error::FileSystem { source, path } => Self::Internal {
                message: format!("failed to read file '{path}', error: {source}"),
            },

            engine::error::Error::InvalidFormat { format } => Self::BadReq {
                message: format!("invalid format: {format}"),
            },

            engine::error::Error::DuckDBNextRow { source } => Self::Internal {
                message: format!("failed to get next row, error: {source}"),
            },

            engine::error::Error::Config { .. } => unreachable!(),

            engine::error::Error::LibSQLConnection { source } => Self::Internal {
                message: format!("failed to connect to libsql, error: {source}"),
            },

            engine::error::Error::LibSQLExecute { source, sql } => Self::Internal {
                message: format!("failed to execute sql '{sql}', error: {source}"),
            },

            engine::error::Error::LibSQLNextRow { source } => Self::Internal {
                message: format!("failed to get next row, error: {source}"),
            },

            engine::error::Error::LibSQLConverstion { message } => Self::Internal {
                message: format!("failed to convert value, error: {message}"),
            },

            engine::error::Error::LibSQLPrepareStatement { source, sql } => Self::BadReq {
                message: format!("prepare statement error: {source}, sql: {sql}"),
            },

            engine::error::Error::DuckDBPool { source } => Self::Internal {
                message: format!("failed to get duckdb connection from pool, error: {source}"),
            },
            _ => Self::Internal {
                message: "internal error".to_string(),
            },
        }
    }
}
