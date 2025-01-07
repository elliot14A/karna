use gloo_net::http::Request;
use snafu::ResultExt;

use crate::common::models::Dataset;
use crate::actions::error::{Result, SendRequestSnafu};

pub async fn details(dataset_id: &str)-> Result<Dataset> {
    let response = Request::get(&format!("/api/datasets/{}", dataset_id))
        .send()
        .await
        .context(SendRequestSnafu)?;

    response.json().await.context(SendRequestSnafu)
}
