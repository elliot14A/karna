use crate::error::Result;
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Json};
use engine::{driver::DatasetStore, models};
use std::sync::Arc;

pub async fn update<S: DatasetStore>(
    Extension(store): Extension<Arc<S>>,
    Path(id): Path<String>,
    Json(dataset): Json<models::UpdateDataset>,
) -> Result<impl IntoResponse> {
    let dataset =
        store
            .update(id.clone(), dataset)
            .await?
            .ok_or_else(|| crate::error::Error::NotFound {
                message: format!("Dataset with id {} not found", id),
            })?;
    Ok((StatusCode::OK, Json(dataset)).into_response())
}
