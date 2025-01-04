use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Dataset {
    pub id: String,
    pub name: String,
    pub file_name: String,
    pub r#type: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub row_count: u64,
    pub size: u64,
}

#[derive(Debug, Serialize)]
pub struct CreateDataset {
    pub name: String,
    pub file_name: String,
    pub r#type: String,
    pub description: Option<String>,
    pub row_count: u64,
    pub size: u64,
}

#[derive(Debug, Serialize)]
pub struct UpdateDataset {
    pub description: Option<String>,
}
