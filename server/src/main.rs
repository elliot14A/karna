#![allow(dead_code)]

use std::sync::Arc;
use tower_http::limit::RequestBodyLimitLayer;

use api::middleware::create_logger_middleware;
use app::*;
use axum::{extract::DefaultBodyLimit, Extension, Router};
use engine::{
    driver::{
        duckdb::{config::Config, driver::DuckDBDriver},
        libsql::driver::LibSQLDriver,
    },
    sources::file_system::FileSystem,
};
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, Level};

mod api;
mod error;
mod fileserv;

// Set GB as the body limit
const GB: usize = 1024 * 1024 * 1024;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any)
        .expose_headers(Any);

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // get current directory
    let path = std::env::current_dir().unwrap();
    let path = path.join("./karna/main.db");

    // Initialize the duckdb driver
    let config_res = Config::new(path);
    if config_res.is_err() {
        panic!("Failed to create config: {:?}", config_res.err());
    }
    let config = config_res.unwrap();
    let duckdb_res = DuckDBDriver::new(config);
    if duckdb_res.is_err() {
        panic!("Failed to create duckdb driver: {:?}", duckdb_res.err());
    }
    let duckdb = duckdb_res.unwrap();

    // Initialize the libsql driver
    let path = "./karna/data/karna.db";
    let libsql_res = LibSQLDriver::new(path).await;
    if libsql_res.is_err() {
        panic!("Failed to create libsql driver: {:?}", libsql_res.err());
    }
    let libsql = libsql_res.unwrap();

    // Initialize file system source
    let file_system = FileSystem::new();

    // build our application with a route
    let app = Router::new()
        .layer(cors)
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let app = app
        .nest("/api", api::routes())
        .layer(Extension(Arc::new(duckdb)))
        .layer(Extension(Arc::new(libsql)))
        .layer(Extension(Arc::new(file_system)))
        .layer(create_logger_middleware())
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(GB));

    // let app = app.layer(Extension(Arc::new()));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("🌞 karna is running on {}", addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
