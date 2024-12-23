use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnResponse, HttpMakeClassifier, TraceLayer,
};
use tracing::Level;

pub fn create_logger_middleware() -> TraceLayer<HttpMakeClassifier> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
        .on_failure(DefaultOnFailure::new().level(Level::ERROR))
}
