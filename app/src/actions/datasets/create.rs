use crate::actions::error::{ActionError, ParseResponseSnafu, SendRequestSnafu};

use crate::actions::error::Result;
use crate::common::models::Dataset;
use gloo_net::http::Request;
use snafu::ResultExt;
use web_sys::FormData;

pub async fn upload_file_system(file: web_sys::File) -> Result<Dataset> {
    let form_data = FormData::new().map_err(|e| ActionError::CreateFormData {
        message: format!("Failed to conver file to FormData: {:?}", e.as_string()),
    })?;
    form_data
        .append_with_blob("file", &file)
        .map_err(|e| ActionError::CreateFormData {
            message: format!("Failed to append file to FormData: {:?}", e.as_string()),
        })?;

    let request = Request::post("/api/datasets/upload/file_system")
        .body(&form_data)
        .context(SendRequestSnafu)?
        .send()
        .await
        .context(SendRequestSnafu)?;

    let dataset = request.json().await.context(ParseResponseSnafu)?;

    Ok(dataset)
}
