use std::path::Path;

use crate::error::{MigrationDirNotFoundSnafu, Result, SqlxConnectionSnafu, SqlxMigrationSnafu};
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
}
