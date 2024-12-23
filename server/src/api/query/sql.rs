use std::sync::Arc;

use crate::error::{BadReqSnafu, Result};
use axum::{response::IntoResponse, Extension, Json};
use engine::driver::OlapDriver;

#[derive(serde::Deserialize)]
pub struct Request {
    query: String,
}

pub async fn sql<O: OlapDriver>(
    Extension(olap): Extension<Arc<O>>,
    Json(request): Json<Request>,
) -> Result<impl IntoResponse> {
    if request.query.is_empty() {
        return BadReqSnafu {
            message: "Query is empty".to_string(),
        }
        .fail();
    }
    let result = olap.query(&request.query).await?;
    Ok(Json(result))
}
