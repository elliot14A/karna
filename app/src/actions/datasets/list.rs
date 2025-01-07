use gloo_net::http::Request;
use snafu::ResultExt;

use crate::{
    actions::error::{ParseResponseSnafu, Result, SendRequestSnafu},
    common::models::Dataset,
};

pub async fn list() -> Result<Vec<Dataset>> {
    let response = Request::get("/api/datasets")
        .send()
        .await
        .context(SendRequestSnafu)?;

     response.json().await.context(ParseResponseSnafu)
}
