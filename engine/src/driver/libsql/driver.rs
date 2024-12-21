use crate::driver::DatasetStore;
use crate::error::{
    Error, LibSQLConnectionSnafu, LibSQLExecuteSnafu, LibSQLNextRowSnafu,
    LibSQLPrepareStatementSnafu, Result,
};
use crate::models::{self, CreateDataset, Dataset, UpdateDataset};
use async_trait::async_trait;
use libsql::{de, params, Builder, Connection, Database};
use snafu::ResultExt;
use std::path::Path;

pub struct LibSQLDriver {
    conn: Connection,
}

impl LibSQLDriver {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db: Database = Builder::new_local(path)
            .build()
            .await
            .context(LibSQLConnectionSnafu)?;
        let conn = db.connect().context(LibSQLConnectionSnafu)?;
        let driver = Self { conn };
        driver.migrate().await?;
        Ok(driver)
    }

    pub async fn migrate(&self) -> Result<()> {
        const SQL: &str = r#"
            create table if not exists dataset (
                id text primary key not null unique,
                name text not null,
                file_name text not null,
                type text not null,
                description text,
                created_at string not null default current_timestamp,
                updated_at string not null default current_timestamp,
                row_count integer not null,
                size integer not null
            );

            create trigger if not exists dataset_updated_at_trigger
            after update on dataset
            begin 
                update dataset
                set updated_at = datetime('now')
                where id = NEW.id;
            end;
       "#;

        self.conn
            .execute(SQL, params!())
            .await
            .context(LibSQLExecuteSnafu { sql: SQL })?;

        Ok(())
    }

    pub async fn create_dataset(&self, input: CreateDataset) -> Result<Dataset> {
        const SQL: &str = r#"
            insert into dataset (id, name, file_name, type, description, row_count, size)
            values (?, ?, ?, ?, ?, ?, ?)
            returning *;
            "#;

        let mut stmt = self.prepare_statement(SQL).await?;
        let uuid = uuid::Uuid::new_v4().to_string();

        let row = stmt
            .query(params![
                uuid,
                input.name,
                input.file_name,
                input.r#type,
                input.description,
                input.row_count,
                input.size
            ])
            .await
            .context(LibSQLExecuteSnafu { sql: SQL })?
            .next()
            .await
            .context(LibSQLNextRowSnafu)?
            .unwrap();

        self.convert_row_to_dataset(row)
    }

    pub async fn get_dataset_by_id(&self, id: String) -> Result<Option<Dataset>> {
        const SQL: &str = "select * from dataset where id = ?;";

        let mut stmt = self.prepare_statement(SQL).await?;
        let row = stmt
            .query(params![id])
            .await
            .context(LibSQLExecuteSnafu { sql: SQL })?
            .next()
            .await
            .context(LibSQLNextRowSnafu)?;

        match row {
            Some(row) => Ok(Some(self.convert_row_to_dataset(row)?)),
            None => Ok(None),
        }
    }

    pub async fn list_datasets(&self) -> Result<Vec<Dataset>> {
        const SQL: &str = "select * from dataset;";

        let mut stmt = self.prepare_statement(SQL).await?;
        let mut datasets = Vec::new();
        let mut rows = stmt
            .query(params![])
            .await
            .context(LibSQLExecuteSnafu { sql: SQL })?;

        while let Some(row) = rows.next().await.context(LibSQLNextRowSnafu)? {
            datasets.push(self.convert_row_to_dataset(row)?);
        }

        Ok(datasets)
    }

    pub async fn delete_dataset(&self, id: String) -> Result<()> {
        const SQL: &str = "delete from dataset where id = ?;";

        let mut stmt = self.prepare_statement(SQL).await?;
        stmt.execute(params![id])
            .await
            .context(LibSQLExecuteSnafu { sql: SQL })?;

        Ok(())
    }

    pub async fn update_dataset(&self, input: UpdateDataset) -> Result<Option<Dataset>> {
        const SQL: &str = "update dataset set description = ? where id = ? returning *;";

        let mut stmt = self.prepare_statement(SQL).await?;
        let row = stmt
            .query(params![input.description, input.id])
            .await
            .context(LibSQLExecuteSnafu { sql: SQL })?
            .next()
            .await
            .context(LibSQLNextRowSnafu)?;

        if let Some(row) = row {
            return Ok(Some(self.convert_row_to_dataset(row)?));
        }
        Ok(None)
    }

    async fn prepare_statement(&self, sql: &str) -> Result<libsql::Statement> {
        self.conn
            .prepare(sql)
            .await
            .context(LibSQLPrepareStatementSnafu { sql })
    }

    fn convert_row_to_dataset(&self, row: libsql::Row) -> Result<Dataset> {
        de::from_row::<Dataset>(&row).map_err(|e| Error::LibSQLConverstion {
            message: e.to_string(),
        })
    }
}

#[async_trait]
impl DatasetStore for LibSQLDriver {
    async fn create(&self, dataset: CreateDataset) -> Result<Dataset> {
        self.create_dataset(dataset).await
    }

    async fn details(&self, id: String) -> Result<Option<Dataset>> {
        self.get_dataset_by_id(id).await
    }

    async fn update(&self, dataset: UpdateDataset) -> Result<Option<models::Dataset>> {
        self.update_dataset(dataset).await
    }

    async fn delete(&self, id: String) -> Result<()> {
        self.delete_dataset(id).await
    }

    async fn list(&self) -> Result<Vec<Dataset>> {
        self.list_datasets().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use uuid::Uuid;

    async fn setup_test_db() -> Result<(LibSQLDriver, String)> {
        let temp_dir = env::temp_dir();
        let db_name = format!("test_{}.db", Uuid::new_v4());
        let db_path = temp_dir.join(db_name);

        let driver = LibSQLDriver::new(&db_path).await?;
        driver.migrate().await?;

        Ok((driver, db_path.to_string_lossy().to_string()))
    }

    async fn cleanup_test_db(db_path: &str) {
        // Clean up the test database
        if let Err(e) = std::fs::remove_file(db_path) {
            eprintln!("Failed to clean up test database: {}", e);
        }
    }

    #[tokio::test]
    async fn test_create_dataset() -> Result<()> {
        let (driver, db_path) = setup_test_db().await?;

        let input = CreateDataset {
            name: "Test Dataset".to_string(),
            file_name: "test.csv".to_string(),
            r#type: "csv".to_string(),
            description: Some("Test description".to_string()),
            row_count: 100,
            size: 1024,
        };

        let dataset = driver.create_dataset(input).await?;

        assert!(!dataset.id.is_empty(), "ID is empty");
        assert_eq!(dataset.name, "Test Dataset", "Name mismatch");
        assert_eq!(dataset.file_name, "test.csv", "File name mismatch");
        assert_eq!(dataset.r#type, "csv", "Type mismatch");
        assert_eq!(
            dataset.description,
            Some("Test description".to_string()),
            "Description mismatch"
        );
        assert_eq!(dataset.row_count, 100, "Row count mismatch");
        assert_eq!(dataset.size, 1024, "Size mismatch");

        cleanup_test_db(&db_path).await;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_dataset_by_id() -> Result<()> {
        let (driver, db_path) = setup_test_db().await?;

        // First create a dataset
        let input = CreateDataset {
            name: "Test Dataset".to_string(),
            file_name: "test.csv".to_string(),
            r#type: "csv".to_string(),
            description: Some("Test description".to_string()),
            row_count: 100,
            size: 1024,
        };

        let created_dataset = driver.create_dataset(input).await?;

        // Then retrieve it
        let retrieved_dataset = driver.get_dataset_by_id(created_dataset.id.clone()).await?;
        assert!(retrieved_dataset.is_some(), "Dataset not found");

        let retrieved_dataset = retrieved_dataset.unwrap();
        assert_eq!(retrieved_dataset.id, created_dataset.id, "ID mismatch");
        assert_eq!(
            retrieved_dataset.name, created_dataset.name,
            "Name mismatch"
        );

        // Test non-existent dataset
        let non_existent = driver
            .get_dataset_by_id("non-existent-id".to_string())
            .await?;
        assert!(non_existent.is_none());

        cleanup_test_db(&db_path).await;
        Ok(())
    }

    #[tokio::test]
    async fn test_list_datasets() -> Result<()> {
        let (driver, db_path) = setup_test_db().await?;

        // Create multiple datasets
        let datasets = vec![
            CreateDataset {
                name: "Dataset 1".to_string(),
                file_name: "test1.csv".to_string(),
                r#type: "csv".to_string(),
                description: Some("Description 1".to_string()),
                row_count: 100,
                size: 1024,
            },
            CreateDataset {
                name: "Dataset 2".to_string(),
                file_name: "test2.csv".to_string(),
                r#type: "csv".to_string(),
                description: Some("Description 2".to_string()),
                row_count: 200,
                size: 2048,
            },
        ];

        for dataset in datasets {
            driver.create_dataset(dataset).await?;
        }

        let listed_datasets = driver.list_datasets().await?;
        assert_eq!(listed_datasets.len(), 2, "Incorrect number of datasets");

        cleanup_test_db(&db_path).await;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_dataset() -> Result<()> {
        let (driver, db_path) = setup_test_db().await?;

        // First create a dataset
        let input = CreateDataset {
            name: "Test Dataset".to_string(),
            file_name: "test.csv".to_string(),
            r#type: "csv".to_string(),
            description: Some("Original description".to_string()),
            row_count: 100,
            size: 1024,
        };

        let created_dataset = driver.create_dataset(input).await?;

        // Update the description
        let update_input = UpdateDataset {
            id: created_dataset.id.clone(),
            description: Some("Updated description".to_string()),
        };

        let updated_dataset = driver.update_dataset(update_input).await?;
        assert!(updated_dataset.is_some(), "Update failed");
        let updated_dataset = updated_dataset.unwrap();
        assert_eq!(
            updated_dataset.description,
            Some("Updated description".to_string()),
            "Description not updated"
        );
        assert_eq!(updated_dataset.id, created_dataset.id, "ID changed");

        cleanup_test_db(&db_path).await;
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_dataset() -> Result<()> {
        let (driver, db_path) = setup_test_db().await?;

        // First create a dataset
        let input = CreateDataset {
            name: "Test Dataset".to_string(),
            file_name: "test.csv".to_string(),
            r#type: "csv".to_string(),
            description: Some("Test description".to_string()),
            row_count: 100,
            size: 1024,
        };

        let created_dataset = driver.create_dataset(input).await?;

        // Delete the dataset
        driver.delete_dataset(created_dataset.id.clone()).await?;

        // Verify it's deleted
        let deleted_dataset = driver.get_dataset_by_id(created_dataset.id).await?;
        assert!(deleted_dataset.is_none(), "Dataset not deleted");

        cleanup_test_db(&db_path).await;
        Ok(())
    }

    #[tokio::test]
    async fn test_error_cases() -> Result<()> {
        let (driver, db_path) = setup_test_db().await?;

        // Test duplicate ID insertion (should fail)
        let input = CreateDataset {
            name: "Test Dataset".to_string(),
            file_name: "test.csv".to_string(),
            r#type: "csv".to_string(),
            description: Some("Test description".to_string()),
            row_count: 100,
            size: 1024,
        };

        driver.create_dataset(input).await?;

        // Attempt to update non-existent dataset
        let update_result = driver
            .update_dataset(UpdateDataset {
                id: "non-existent-id".to_string(),
                description: Some("New description".to_string()),
            })
            .await?;
        assert!(update_result.is_none(), "Update should fail");

        cleanup_test_db(&db_path).await;
        Ok(())
    }
}
