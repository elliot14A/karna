use std::collections::HashMap;

use gloo_net::http::Request;
use serde_json::Value;
use snafu::ResultExt;

use crate::actions::error::Result;

use super::error::{ParseResponseSnafu, SendRequestSnafu};

pub async fn query_dataset_with_pagination(dataset: &str, page: u16, limit: u16) -> Result<Vec<HashMap<String, Value>>> {
    let offset = if page == 1 { 0 } else { (page - 1) * limit }; 

    let sql = format!("select * from {} limit {} offset {}", dataset, limit, offset);

    let response = Request::post("/api/query/sql")
        .json(&serde_json::json!({ "query": sql }))
        .context(SendRequestSnafu)?
        .send()
        .await
        .context(SendRequestSnafu)?;

    let data = response.json().await.context(ParseResponseSnafu)?;

    leptos::logging::log!("data: {:?}", data);


    Ok(data)
}
