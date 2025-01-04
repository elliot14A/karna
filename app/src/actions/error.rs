use gloo_net::Error;
use snafu::Snafu;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum ActionError {
    #[snafu(display("Failed to create FormData: {}", message))]
    CreateFormData { message: String },

    #[snafu(display("Failed to send request: {}", source))]
    SendRequest { source: Error },

    #[snafu(display("Failed to parse response: {}", source))]
    ParseResponse { source: Error },
}

pub type Result<T> = std::result::Result<T, ActionError>;
