use std::sync::Arc;

use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Json};
use engine::driver::DatasetStore;

use crate::error::{Error, Result};

pub async fn details<S: DatasetStore>(
    Extension(store): Extension<Arc<S>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let dataset = store
        .details(id.clone())
        .await?
        .ok_or_else(|| Error::NotFound {
            message: format!("Dataset with id {} not found", id),
        })?;

    Ok((StatusCode::OK, Json(dataset)).into_response())
}
