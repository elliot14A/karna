use std::sync::Arc;

use crate::error::Result;
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use engine::driver::DatasetStore;

pub async fn list<S: DatasetStore>(
    Extension(store): Extension<Arc<S>>,
) -> Result<impl IntoResponse> {
    let datasets = store.list().await?;
    Ok((StatusCode::OK, Json(datasets)).into_response())
}
