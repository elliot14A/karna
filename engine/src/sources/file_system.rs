use crate::error::{Error, FileSystemSnafu, InvalidFormatSnafu, Result};
use snafu::ResultExt;
use std::{collections::HashMap, path::Path};
use tracing::{debug, error, info};

use super::utils::source_reader;

pub mod constants {
    pub const DELIMITER: &str = "delimiter";
    pub const HAS_HEADER: &str = "header";
    pub const SAMPLE_SIZE: &str = "sample_size";
    pub const ALL_VARCHAR: &str = "all_varchar";
    pub const AUTO_DETECT: &str = "auto_detect";
    pub const MAXIMUM_LINE_SIZE: &str = "maximum_line_size";
    pub const COMPRESSION: &str = "compression";
    pub const UNION_BY_NAME: &str = "union_by_name";

    pub const DEFAULT_SAMPLE_SIZE: &str = "1000";
    pub const DEFAULT_MAX_LINE_SIZE: &str = "104857600"; // 100MB
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileFormat {
    Csv,
    Tsv,
    Txt,
    Parquet,
    Json,
}

impl FileFormat {
    fn from_extension(extension: &str) -> Result<Self> {
        match extension.to_lowercase().as_str() {
            "csv" => {
                info!("📊 Detected CSV format");
                Ok(FileFormat::Csv)
            }
            "tsv" => {
                info!("📑 Detected TSV format");
                Ok(FileFormat::Tsv)
            }
            "txt" => {
                info!("📝 Detected TXT format");
                Ok(FileFormat::Txt)
            }
            "parquet" => {
                info!("📦 Detected Parquet format");
                Ok(FileFormat::Parquet)
            }
            "json" => {
                info!("🔍 Detected JSON format");
                Ok(FileFormat::Json)
            }
            _ => {
                error!("❌ Invalid file format: {}", extension);
                InvalidFormatSnafu {
                    format: extension.to_string(),
                }
                .fail()
            }
        }
    }

    fn default_params(&self) -> HashMap<String, String> {
        debug!("🔧 Initializing parameters for format: {:?}", self);
        use constants::*;
        let mut params = HashMap::new();

        // Common parameters for text-based formats
        match self {
            FileFormat::Csv | FileFormat::Tsv | FileFormat::Txt | FileFormat::Json => {
                debug!("📝 Setting common parameters for text-based format");
                params.insert(AUTO_DETECT.to_string(), "true".to_string());
                params.insert(SAMPLE_SIZE.to_string(), DEFAULT_SAMPLE_SIZE.to_string());
            }
            _ => {}
        }

        // Format-specific parameters
        match self {
            FileFormat::Csv | FileFormat::Tsv => {
                debug!("📊 Setting CSV/TSV specific parameters");
                params.insert(HAS_HEADER.to_string(), "true".to_string());

                if matches!(self, FileFormat::Tsv) {
                    debug!("📑 Setting TSV delimiter");
                    params.insert(DELIMITER.to_string(), "\t".to_string());
                }
            }
            FileFormat::Json => {
                debug!("📋 Setting JSON specific parameters");
                params.insert(
                    MAXIMUM_LINE_SIZE.to_string(),
                    DEFAULT_MAX_LINE_SIZE.to_string(),
                );
            }
            FileFormat::Parquet => {
                debug!("📦 Setting Parquet specific parameters");
                params.insert(UNION_BY_NAME.to_string(), "true".to_string());
            }
            _ => {}
        }

        debug!("⚙️ Configured parameters: {:?}", params);
        params
    }
}

#[derive(Debug, Default, Clone)]
pub struct FileSystem {
    /// Maximum file size in MB (multiple of 1024)
    max_file_size: Option<usize>,
}

impl FileSystem {
    pub fn new() -> Self {
        debug!("🆕 Creating new FileSystem instance");
        Self::default()
    }

    pub fn with_max_file_size(mut self, max_file_size: usize) -> Self {
        info!("📏 Setting max file size to {} MB", max_file_size);
        self.max_file_size = Some(max_file_size);
        self
    }

    pub fn validate<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        let path = file_path.as_ref();
        let path_str = path.display().to_string();
        info!("🔍 Validating file: {}", path_str);

        let metadata = std::fs::metadata(path).context(FileSystemSnafu {
            path: path_str.clone(),
        })?;

        if metadata.len() == 0 {
            error!("⚠️ Empty file detected: {}", path_str);
            return Err(Error::FileSystem {
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, "empty file"),
                path: path_str,
            });
        }

        if let Some(max_size) = self.max_file_size {
            let max_bytes = max_size * 1024 * 1024;
            debug!("📊 Checking file size limit: {} bytes", max_bytes);
            if metadata.len() > max_bytes as u64 {
                error!("⚠️ File size exceeds limit: {} MB", max_size);
                return Err(Error::FileSystem {
                    source: std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("file size exceeds limit of {} MB", max_size),
                    ),
                    path: path_str,
                });
            }
        }

        info!("✅ File validation successful: {}", path_str);
        Ok(())
    }

    pub fn generate_sql<P: AsRef<Path>>(
        &self,
        file_path: P,
        ingestion_params: HashMap<String, String>,
    ) -> Result<String> {
        let path = file_path.as_ref();
        debug!("🔨 Generating SQL for file: {}", path.display());

        let extension = path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or_else(|| {
                error!("❌ Invalid file extension: {}", path.display());
                Error::FileSystem {
                    source: std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "invalid extension",
                    ),
                    path: path.display().to_string(),
                }
            })?;

        let format = FileFormat::from_extension(extension)?;
        debug!("📄 File format determined: {:?}", format);

        // Merge default parameters with ingestion parameters
        let mut final_params = format.default_params();
        for (key, value) in ingestion_params {
            debug!("🔄 Overriding parameter: {} = {}", key, value);
            final_params.insert(key, value);
        }

        info!("⚙️ Generating SQL with parameters: {:?}", final_params);
        // Generate SQL using source_reader
        let sql = source_reader(
            path.to_str().ok_or_else(|| {
                error!("❌ Invalid path: {}", path.display());
                Error::FileSystem {
                    source: std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid path"),
                    path: path.display().to_string(),
                }
            })?,
            extension,
            final_params,
        )?;

        let sql = format!("select * from {sql}");

        debug!("✅ Generated SQL query: {}", sql);
        Ok(sql)
    }
}
