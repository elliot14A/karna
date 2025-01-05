use std::sync::Arc;

use crate::error::{ Error,  Result};
use axum::{extract::Path, response::IntoResponse, Extension};
use engine::driver::{DatasetStore, OlapDriver};

pub async fn delete<O: OlapDriver,S: DatasetStore>(
    Extension(store): Extension<Arc<S>>,
    Extension(olap): Extension<Arc<O>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let dataset = store.details(id.clone()).await?
        .ok_or_else(|| Error::NotFound{message: format!("Dataset with id {} not found", id)})?;
    olap.drop_table(&dataset.name).await?;
    Ok(store.delete(id).await?)
}
