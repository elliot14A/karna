use axum::{routing::post, Router};

mod graphql;
mod rest;
mod sql;

use engine::driver::OlapDriver;
use sql::sql;

pub fn router<O: OlapDriver>() -> Router {
    Router::new().route("/sql", post(sql::<O>))
}
