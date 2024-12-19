use snafu::OptionExt;

use crate::error::{ConfigSnafu, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Config {
    dsn: String,
    cpu_cores: Option<usize>,
    memory_limit_gb: Option<usize>,
    storage_limit_bytes: Option<usize>,
    boot_queries: Vec<String>,
    db_file_path: PathBuf,
    db_storage_path: PathBuf,
}

impl Config {
    pub fn new<P: AsRef<Path>>(dsn: P) -> Result<Self> {
        let dsn_path = Path::new(dsn.as_ref());

        // Validate DSN path
        if !dsn_path.is_absolute() {
            return ConfigSnafu {
                message: "DSN path must be absolute".to_string(),
            }
            .fail();
        }

        // Extract storage path and database file path
        let db_storage_path = dsn_path.parent().context(ConfigSnafu {
            message: "DSN path has no parent directory",
        })?;

        Ok(Self {
            dsn: dsn_path.to_string_lossy().into_owned(),
            cpu_cores: None,
            memory_limit_gb: None,
            storage_limit_bytes: None,
            boot_queries: Vec::new(),
            db_file_path: dsn_path.to_path_buf(),
            db_storage_path: db_storage_path.to_path_buf(),
        })
    }

    /// Builds the complete DSN string with all configuration parameters
    pub fn build_dsn(&self) -> String {
        let mut params = vec![];

        if let Some(cores) = self.cpu_cores {
            params.push(format!("cpu={cores}"));
        }

        if let Some(memory) = self.memory_limit_gb {
            params.push(format!("max_memory_limit={memory}"));
        }

        if let Some(storage) = self.storage_limit_bytes {
            params.push(format!("storage_limit={storage}"));
        }

        if params.is_empty() {
            self.dsn.clone()
        } else {
            format!("{}?{}", self.dsn, params.join("&"))
        }
    }

    // Builder methods

    /// Sets the number of CPU cores to use
    ///
    /// # Arguments
    /// * `cores` - Number of CPU cores
    ///
    /// # Returns
    /// * `Result<Self, ConfigError>` - Updated config or error if invalid
    pub fn with_cpu_cores(mut self, cores: usize) -> Result<Self> {
        let available_cores = num_cpus::get();
        if cores == 0 || cores > available_cores {
            return ConfigSnafu {
                message: format!(
                    "Invalid CPU core count: {} (available: {})",
                    cores, available_cores
                ),
            }
            .fail();
        }
        self.cpu_cores = Some(cores);
        Ok(self)
    }

    /// Sets the memory limit in gigabytes
    pub fn with_memory_limit_gb(mut self, limit_gb: usize) -> Result<Self> {
        if !(1..=1024).contains(&limit_gb) {
            return ConfigSnafu {
                message: format!("Invalid memory limit: {} GB", limit_gb),
            }
            .fail();
        }
        self.memory_limit_gb = Some(limit_gb);
        Ok(self)
    }

    /// Sets the storage limit in bytes
    pub fn with_storage_limit_bytes(mut self, limit_bytes: usize) -> Result<Self> {
        const MIN_STORAGE: usize = 1024 * 1024; // 1MB minimum
        if limit_bytes < MIN_STORAGE {
            return ConfigSnafu {
                message: format!("Invalid storage limit: {} bytes", limit_bytes),
            }
            .fail();
        }
        self.storage_limit_bytes = Some(limit_bytes);
        Ok(self)
    }

    /// Adds a boot query to the configuration
    pub fn with_boot_query<S: Into<String>>(mut self, query: S) -> Self {
        self.boot_queries.push(query.into());
        self
    }

    // Getters
    pub fn dsn(&self) -> &str {
        &self.dsn
    }

    pub fn cpu_cores(&self) -> Option<usize> {
        self.cpu_cores
    }

    pub fn memory_limit_gb(&self) -> Option<usize> {
        self.memory_limit_gb
    }

    pub fn storage_limit_bytes(&self) -> Option<usize> {
        self.storage_limit_bytes
    }

    pub fn boot_queries(&self) -> &[String] {
        &self.boot_queries
    }

    pub fn db_file_path(&self) -> &Path {
        &self.db_file_path
    }

    pub fn db_storage_path(&self) -> &Path {
        &self.db_storage_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::*;
    use snafu::ResultExt;
    use tokio::fs;

    #[tokio::test]
    async fn test_error_handling() {
        let path = "/nonexistent/path";
        let result = fs::read_to_string(path).await.context(FileSystemSnafu {
            path: path.to_string(),
        });

        match result {
            Err(Error::FileSystem {
                path: error_path, ..
            }) => {
                assert_eq!(error_path, path);
            }
            _ => panic!("Expected FileSystem error"),
        }

        let config_result = Config::new("relative/path");
        assert!(matches!(config_result.unwrap_err(), Error::Config { .. }));
    }

    #[tokio::test]
    async fn test_config_builder() -> Result<()> {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("test.db").to_str().unwrap().to_owned();

        let config = Config::new(&test_path)?
            .with_cpu_cores(1)?
            .with_memory_limit_gb(2)?
            .with_storage_limit_bytes(2 * 1024 * 1024)?;

        assert_eq!(config.cpu_cores, Some(1));
        assert_eq!(config.memory_limit_gb, Some(2));
        assert_eq!(config.storage_limit_bytes, Some(2 * 1024 * 1024));

        Ok(())
    }
}
