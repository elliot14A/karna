use std::sync::Arc;

use crate::error::Result;
use axum::{extract::Path, response::IntoResponse, Extension};
use engine::driver::DatasetStore;

pub async fn delete<S: DatasetStore>(
    Extension(store): Extension<Arc<S>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    Ok(store.delete(id).await?)
}
