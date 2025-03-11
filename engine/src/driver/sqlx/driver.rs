use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    driver::DatasetStore,
    error::{
        MigrationDirNotFoundSnafu, Result, SqlxConnectionSnafu, SqlxExecuteSnafu,
        SqlxMigrationSnafu,
    },
    models::{CreateDataset, Dataset, UpdateDataset},
};
use async_trait::async_trait;
use snafu::ResultExt;
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Pool, Sqlite};
use tracing::{debug, error, info};

pub struct SqlxDriver {
    pool: Pool<Sqlite>,
}

impl SqlxDriver {
    pub async fn new<P: AsRef<Path>>(db_path: P, migration_dir_path: P) -> Result<Self> {
        let db_url = format!("sqlite://{}", db_path.as_ref().to_str().unwrap());

        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            info!("Creating database at {}", db_path.as_ref().display());
            Sqlite::create_database(&db_url)
                .await
                .context(SqlxConnectionSnafu)?;
        }

        let pool = SqlitePool::connect(&db_url)
            .await
            .context(SqlxConnectionSnafu)?;

        let driver = SqlxDriver { pool };

        let migration_path = migration_dir_path.as_ref();
        if !migration_path.exists() || !migration_path.is_dir() {
            error!(
                "Migration path does not exist or is not a directory: {}",
                migration_path.display()
            );
            return MigrationDirNotFoundSnafu {
                path: migration_path.display().to_string(),
            }
            .fail();
        }

        let migrator = sqlx::migrate::Migrator::new(migration_path)
            .await
            .context(SqlxMigrationSnafu)?;
        migrator
            .run(&driver.pool)
            .await
            .context(SqlxMigrationSnafu)?;

        driver.optimize_connection().await?;

        Ok(driver)
    }

    async fn optimize_connection(&self) -> Result<()> {
        let pragmas = [
            "PRAGMA journal_mode = WAL;",
            "PRAGMA synchronous = NORMAL;",
            "PRAGMA foreign_keys = ON;",
            "PRAGMA busy_timeout = 5000;",
        ];

        for pragma in pragmas {
            debug!("Running pragma: {}", pragma);
            sqlx::query(pragma)
                .execute(&self.pool)
                .await
                .context(crate::error::SqlxExecuteSnafu { sql: pragma })?;
        }

        Ok(())
    }

    pub async fn create_dataset(&self, input: CreateDataset) -> Result<Dataset> {
        let uuid = uuid::Uuid::new_v4().to_string();
        let row_count = input.row_count as i64;
        let size = input.size as i64;

        let res = sqlx::query!(
            r#"
                insert into dataset (id, name, file_name, type, description, row_count, size)
                values (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                returning *
            "#,
            uuid,
            input.name,
            input.file_name,
            input.r#type,
            input.description,
            row_count,
            size,
        )
        .fetch_one(&self.pool)
        .await
        .context(SqlxExecuteSnafu {
            sql: "insert into dataset".to_string(),
        })?;

        Ok(Dataset {
            id: res.id,
            name: res.name,
            file_name: res.file_name,
            r#type: res.r#type,
            description: res.description,
            created_at: super::parse_datetime_string(&res.created_at)?,
            updated_at: super::parse_datetime_string(&res.updated_at)?,
            row_count: res.row_count as u64,
            size: res.size as u64,
        })
    }

    pub async fn get_dataset_by_id(&self, id: String) -> Result<Option<Dataset>> {
        Ok(sqlx::query!("select * from dataset where id = ?1", id)
            .fetch_optional(&self.pool)
            .await
            .context(SqlxExecuteSnafu {
                sql: "select * from dataset".to_string(),
            })?
            .and_then(|d| {
                Some(Dataset {
                    id: d.id,
                    name: d.name,
                    file_name: d.file_name,
                    r#type: d.r#type,
                    description: d.description,
                    created_at: super::parse_datetime_string(&d.created_at).ok()?,
                    updated_at: super::parse_datetime_string(&d.updated_at).ok()?,
                    row_count: d.row_count as u64,
                    size: d.size as u64,
                })
            }))
    }

    pub async fn update_dataset(
        &self,
        id: String,
        input: UpdateDataset,
    ) -> Result<Option<Dataset>> {
        let res = sqlx::query!(
            r#"
                update dataset
                set description = ?1
                where id = ?2
                returning *
            "#,
            input.description,
            id,
        )
        .fetch_optional(&self.pool)
        .await
        .context(SqlxExecuteSnafu {
            sql: "update dataset".to_string(),
        })?;

        Ok(res.map(|d| Dataset {
            id: d.id,
            name: d.name,
            file_name: d.file_name,
            r#type: d.r#type,
            description: d.description,
            created_at: super::parse_datetime_string(&d.created_at).unwrap(),
            updated_at: super::parse_datetime_string(&d.updated_at).unwrap(),
            row_count: d.row_count as u64,
            size: d.size as u64,
        }))
    }

    pub async fn list_datasets(&self) -> Result<Vec<Dataset>> {
        Ok(sqlx::query!("select * from dataset")
            .fetch_all(&self.pool)
            .await
            .context(SqlxExecuteSnafu {
                sql: "select * from dataset".to_string(),
            })?
            .into_iter()
            .map(|d| Dataset {
                id: d.id,
                name: d.name,
                file_name: d.file_name,
                r#type: d.r#type,
                description: d.description,
                created_at: super::parse_datetime_string(&d.created_at).unwrap(),
                updated_at: super::parse_datetime_string(&d.updated_at).unwrap(),
                row_count: d.row_count as u64,
                size: d.size as u64,
            })
            .collect())
    }

    pub async fn delete_dataset(&self, id: String) -> Result<()> {
        sqlx::query!(
            r#"
                delete from dataset
                where id = ?1
                returning *
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await
        .context(SqlxExecuteSnafu {
            sql: "delete from dataset".to_string(),
        })?;

        Ok(())
    }
}

#[async_trait]
impl DatasetStore for SqlxDriver {
    async fn create(&self, dataset: CreateDataset) -> Result<Dataset> {
        self.create_dataset(dataset).await
    }

    async fn details(&self, id: String) -> Result<Option<Dataset>> {
        self.get_dataset_by_id(id).await
    }

    async fn update(&self, id: String, dataset: UpdateDataset) -> Result<Option<Dataset>> {
        self.update_dataset(id, dataset).await
    }

    async fn delete(&self, id: String) -> Result<()> {
        self.delete_dataset(id).await
    }

    async fn list(&self) -> Result<Vec<Dataset>> {
        self.list_datasets().await
    }
}

fn create_temp_dir() -> Result<std::path::PathBuf, std::io::Error> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();

    let temp_dir = std::env::temp_dir().join(format!("sqlx_test_{}", timestamp));
    fs::create_dir_all(&temp_dir)?;
    Ok(temp_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_sqlx_driver_e2e() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_temp_dir()?;
        let db_path = temp_dir.join("test.db");

        let migrations_path = PathBuf::from("./migrations");

        let driver = SqlxDriver::new(&db_path, &migrations_path).await?;

        let create_input = CreateDataset {
            name: "Test Dataset".to_string(),
            file_name: "test.csv".to_string(),
            r#type: "csv".to_string(),
            description: Some("Test dataset description".to_string()),
            row_count: 100,
            size: 1024,
        };

        let created = driver.create(create_input).await?;
        assert_eq!(created.name, "Test Dataset");
        assert_eq!(created.file_name, "test.csv");
        assert_eq!(created.r#type, "csv");
        assert_eq!(
            created.description,
            Some("Test dataset description".to_string())
        );
        assert_eq!(created.row_count, 100);
        assert_eq!(created.size, 1024);

        let dataset_id = created.id.clone();
        let fetched = driver.details(dataset_id.clone()).await?;
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.name, created.name);

        let datasets = driver.list().await?;
        assert_eq!(datasets.len(), 1);
        assert_eq!(datasets[0].id, created.id);

        let update_input = UpdateDataset {
            description: Some("Updated description".to_string()),
        };

        let updated = driver.update(dataset_id.clone(), update_input).await?;
        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.description, Some("Updated description".to_string()));

        driver.delete(dataset_id.clone()).await?;

        let deleted_check = driver.details(dataset_id).await?;
        assert!(deleted_check.is_none());

        let datasets_after_delete = driver.list().await?;
        assert_eq!(datasets_after_delete.len(), 0);

        fs::remove_dir_all(temp_dir)?;

        Ok(())
    }

    #[tokio::test]
    async fn test_sqlx_driver_missing_dataset() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_temp_dir()?;
        let db_path = temp_dir.join("test.db");

        let migrations_path = PathBuf::from("./migrations");

        let driver = SqlxDriver::new(&db_path, &migrations_path).await?;

        let non_existent_id = "non-existent-id".to_string();
        let result = driver.details(non_existent_id.clone()).await?;
        assert!(result.is_none());

        let update_input = UpdateDataset {
            description: Some("This won't work".to_string()),
        };

        let result = driver.update(non_existent_id.clone(), update_input).await?;
        assert!(result.is_none());

        driver.delete(non_existent_id).await?;

        fs::remove_dir_all(temp_dir)?;

        Ok(())
    }

    #[tokio::test]
    async fn test_sqlx_driver_multiple_datasets() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_temp_dir()?;
        let db_path = temp_dir.join("test.db");

        let migrations_path = PathBuf::from("./migrations");

        let driver = SqlxDriver::new(&db_path, &migrations_path).await?;

        let create_inputs = vec![
            CreateDataset {
                name: "Dataset 1".to_string(),
                file_name: "file1.csv".to_string(),
                r#type: "csv".to_string(),
                description: Some("First dataset".to_string()),
                row_count: 100,
                size: 1024,
            },
            CreateDataset {
                name: "Dataset 2".to_string(),
                file_name: "file2.json".to_string(),
                r#type: "json".to_string(),
                description: Some("Second dataset".to_string()),
                row_count: 200,
                size: 2048,
            },
            CreateDataset {
                name: "Dataset 3".to_string(),
                file_name: "file3.parquet".to_string(),
                r#type: "parquet".to_string(),
                description: Some("Third dataset".to_string()),
                row_count: 300,
                size: 4096,
            },
        ];

        let mut created_ids = Vec::new();

        for input in create_inputs {
            let created = driver.create(input).await?;
            created_ids.push(created.id.clone());
        }

        let datasets = driver.list().await?;
        assert_eq!(datasets.len(), 3);

        let types: Vec<String> = datasets.iter().map(|d| d.r#type.clone()).collect();
        assert!(types.contains(&"csv".to_string()));
        assert!(types.contains(&"json".to_string()));
        assert!(types.contains(&"parquet".to_string()));

        for id in created_ids {
            driver.delete(id).await?;
        }

        let datasets_after_delete = driver.list().await?;
        assert_eq!(datasets_after_delete.len(), 0);

        fs::remove_dir_all(temp_dir)?;

        Ok(())
    }

    #[tokio::test]
    async fn test_sqlx_driver_init_failure() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_temp_dir()?;
        let db_path = temp_dir.join("test.db");

        let non_existent_path = PathBuf::from("./non_existent_migrations");

        let result = SqlxDriver::new(&db_path, &non_existent_path).await;
        assert!(result.is_err());

        fs::remove_dir_all(temp_dir)?;

        Ok(())
    }
}
