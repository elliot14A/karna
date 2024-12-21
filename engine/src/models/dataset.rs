use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Dataset {
    pub id: String,
    pub name: String,
    pub file_name: String,
    pub r#type: String,
    pub description: Option<String>,
    #[serde(deserialize_with = "parse_sqlite_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(deserialize_with = "parse_sqlite_datetime")]
    pub updated_at: DateTime<Utc>,
    pub row_count: u64,
    pub size: u64,
}

fn parse_sqlite_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    // Parse using the SQLite format
    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
        .map(|naive| DateTime::from_naive_utc_and_offset(naive, Utc))
        .map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize)]
pub struct CreateDataset {
    pub name: String,
    pub file_name: String,
    pub r#type: String,
    pub description: Option<String>,
    pub row_count: u64,
    pub size: u64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDataset {
    pub id: String,
    pub description: Option<String>,
}
