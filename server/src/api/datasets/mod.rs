use axum::{
    routing::{delete as delete_route, get, patch, post},
    Router,
};
use create::upload_file_system;
use engine::driver::{DatasetStore, OlapDriver};

mod create;
mod delete;
mod details;
mod list;
mod update;

use delete::delete;
use details::details;
use list::list;
use update::update;

pub fn routes<O: OlapDriver, S: DatasetStore>() -> Router {
    Router::new()
        .route("/upload/file_system", post(upload_file_system::<O, S>))
        .route("/", get(list::<S>))
        .nest(
            "/:id",
            Router::new()
                .route("/", get(details::<S>))
                .route("/", patch(update::<S>))
                .route("/", delete_route(delete::<S>)),
        )
}
