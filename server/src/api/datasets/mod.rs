use axum::{routing::post, Router};
use create::upload_file_system;
use engine::driver::{DatasetStore, OlapDriver};

mod create;
mod delete;
mod details;
mod list;
mod update;

pub fn routes<O: OlapDriver, S: DatasetStore>() -> Router {
    Router::new().route("/upload/file_system", post(upload_file_system::<O, S>))
}
