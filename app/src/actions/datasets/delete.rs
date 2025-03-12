use std::future::Future;

use gloo_net::http::Request;
use send_wrapper::SendWrapper;
use snafu::ResultExt;

use crate::actions::error::{Result, SendRequestSnafu};

pub fn delete(dataset_id: String) -> impl Future<Output = Result<()>> + Send {
    SendWrapper::new(async move {
        Request::delete(&format!("/api/datasets/{}", dataset_id))
            .send()
            .await
            .context(SendRequestSnafu)?;
        Ok(())
    })
}
