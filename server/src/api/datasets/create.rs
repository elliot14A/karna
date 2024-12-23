use crate::error::{Error, FileSnafu, MultiPartSnafu, Result};
use axum::{
    extract::{Extension, Multipart},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use engine::{
    driver::{DatasetStore, OlapDriver},
    models::{CreateDataset, Dataset},
    sources::file_system::FileSystem,
};
use serde::Serialize;
use snafu::ResultExt;
use std::{collections::HashMap, os::unix::fs::MetadataExt, path::PathBuf, sync::Arc};
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};
use tracing::{debug, error, info, instrument};

#[derive(Debug, Serialize)]
struct UploadResponse {
    data: Dataset,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[instrument(skip(olap, store, source, multipart))]
pub async fn upload_file_system<O: OlapDriver, S: DatasetStore>(
    Extension(olap): Extension<Arc<O>>,
    Extension(store): Extension<Arc<S>>,
    Extension(source): Extension<Arc<FileSystem>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    info!("Starting file upload process");

    let field = get_multipart_field(&mut multipart).await?;
    let filename = get_filename(&field)?;
    let filepath = std::env::temp_dir().join(&filename);

    debug!("Saving file to: {:?}", filepath);
    save_uploaded_file(field, filepath.clone()).await?;

    let format = get_file_format(&filepath)?;
    let (table_name, row_count) =
        process_file_upload(olap, source, filepath.clone(), filename.clone()).await?;
    let file_size = get_file_size(&filepath)?;

    let dataset = create_dataset(
        store,
        CreateDataset {
            name: table_name,
            size: file_size,
            row_count,
            r#type: format,
            file_name: filename,
            description: None,
        },
    )
    .await?;

    cleanup_temp_file(filepath).await?;

    Ok((StatusCode::CREATED, Json(UploadResponse { data: dataset })).into_response())
}

async fn get_multipart_field(
    multipart: &mut Multipart,
) -> Result<axum::extract::multipart::Field<'_>> {
    match multipart.next_field().await.context(MultiPartSnafu)? {
        Some(field) => Ok(field),
        None => {
            error!("No file found in request");
            return Err(Error::BadReq {
                message: format!("No file found in request"),
            });
        }
    }
}

fn get_filename(field: &axum::extract::multipart::Field<'_>) -> Result<String> {
    match field.file_name() {
        Some(name) => {
            info!("Processing file: {}", name);
            Ok(name.to_string())
        }
        None => {
            error!("No filename provided");
            Err(Error::BadReq {
                message: "No filename provided".to_string(),
            })
        }
    }
}

fn get_file_format(filepath: &PathBuf) -> Result<String> {
    filepath
        .extension()
        .and_then(|ext| ext.to_str())
        .map(String::from)
        .ok_or_else(|| Error::Internal {
            message: "Failed to get file extension".to_string(),
        })
}

fn get_file_size(filepath: &PathBuf) -> Result<u64> {
    std::fs::metadata(filepath)
        .context(FileSnafu {
            message: format!("Failed to get metadata for file at {:?}", filepath),
        })
        .map(|metadata| metadata.size())
}

#[instrument(skip(field))]
async fn save_uploaded_file(
    mut field: axum::extract::multipart::Field<'_>,
    filepath: PathBuf,
) -> Result<()> {
    info!("Starting to save file: {:?}", filepath);

    let file = File::create(&filepath).await.context(FileSnafu {
        message: format!("Failed to create file at {:?}", filepath),
    })?;
    let mut writer = BufWriter::new(file);

    while let Some(chunk) = field.chunk().await.context(MultiPartSnafu)? {
        writer.write_all(&chunk).await.context(FileSnafu {
            message: format!("Failed to write to file at {:?}", filepath),
        })?;
    }

    writer.flush().await.context(FileSnafu {
        message: format!("Failed to flush file at {:?}", filepath),
    })?;

    info!("File saved successfully");
    Ok(())
}

#[instrument(skip(olap, source))]
async fn process_file_upload<O: OlapDriver>(
    olap: Arc<O>,
    source: Arc<FileSystem>,
    filepath: PathBuf,
    filename: String,
) -> Result<(String, u64)> {
    info!("Starting file processing");

    validate_file(&source, &filepath)?;
    let create_sql = generate_sql(&source, &filepath)?;
    let table_name = create_table(&olap, &filename, &create_sql).await?;
    let row_count = get_row_count(&olap, &table_name).await?;

    info!("File processing completed - Row count: {}", row_count);
    Ok((table_name, row_count))
}

fn validate_file(source: &FileSystem, filepath: &PathBuf) -> Result<()> {
    source
        .validate(filepath.clone())
        .map_err(|_| Error::Internal {
            message: format!("Failed to validate file at {:?}", filepath),
        })
}

fn generate_sql(source: &FileSystem, filepath: &PathBuf) -> Result<String> {
    source
        .generate_sql(filepath.clone(), HashMap::new())
        .map_err(|e| Error::Internal {
            message: format!("Failed to generate SQL for file at {:?}: {}", filepath, e),
        })
}

async fn create_table<O: OlapDriver>(
    olap: &Arc<O>,
    filename: &str,
    create_sql: &str,
) -> Result<String> {
    olap.create_table(filename, create_sql)
        .await
        .map_err(|e| Error::Internal {
            message: format!("Failed to create table: {}", e),
        })
}

async fn get_row_count<O: OlapDriver>(olap: &Arc<O>, table_name: &str) -> Result<u64> {
    let sql = format!("SELECT COUNT(*) as count FROM {}", table_name);
    let rows = olap.query(&sql).await.map_err(|e| Error::Internal {
        message: format!("Failed to query table '{}': {:?}", table_name, e),
    })?;

    rows[0]
        .get("count")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| Error::Internal {
            message: "Failed to get row count".to_string(),
        })
}

async fn create_dataset<S: DatasetStore>(store: Arc<S>, input: CreateDataset) -> Result<Dataset> {
    store.create(input).await.map_err(|e| Error::Internal {
        message: format!("Failed to create dataset: {}", e),
    })
}

async fn cleanup_temp_file(filepath: PathBuf) -> Result<()> {
    tokio::fs::remove_file(&filepath).await.context(FileSnafu {
        message: format!("Failed to remove file at {:?}", filepath),
    })
}
