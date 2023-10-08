use actix_web::{
    error::{self},
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::{Display, Error};

/// Represents various possible API errors that can occur during request processing.
///
/// This enum provides a mapping from internal error states to HTTP responses that
/// can be sent back to the client, encapsulating the error's nature and HTTP status code.
#[derive(Debug, Display, Error)]
pub enum ApiError {
    /// Indicates that authentication has not been initiated.
    #[display(fmt = "Authentication not started")]
    AuthNotStarted,

    /// Indicates that a user is not authenticated.
    #[display(fmt = "User not authenticated")]
    UserNotAuth,

    /// Indicates that the provided hash (password) is invalid.
    #[display(fmt = "Invalid password hash")]
    InvalidHash,

    /// Indicates that the user does not exist in the system.
    #[display(fmt = "User does not exist")]
    UserNotExist,

    /// Indicates an error during JSON conversion.
    #[display(fmt = "JSON conversion failed")]
    JsonConvertionFailed,

    /// Indicates that the internal state of the application is unavailable.
    #[display(fmt = "Internal state unavailable")]
    InternalStateUnavailable,

    /// Indicates an error during the proof-of-work check.
    #[display(fmt = "Proof-of-work check failed")]
    PovCheckFailed,
}

impl error::ResponseError for ApiError {
    /// Generates an `HttpResponse` for the current API error.
    ///
    /// This function maps each `ApiError` variant to an appropriate HTTP response,
    /// setting the status code and including the error message in the response body.
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    /// Maps the `ApiError` variant to the corresponding `StatusCode`.
    ///
    /// Each error variant is associated with an HTTP status code that best describes
    /// the nature of the error to the client.
    fn status_code(&self) -> StatusCode {
        match *self {
            ApiError::UserNotExist => StatusCode::NOT_FOUND,
            ApiError::UserNotAuth => StatusCode::UNAUTHORIZED,
            ApiError::JsonConvertionFailed => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InternalStateUnavailable => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InvalidHash => StatusCode::UNAUTHORIZED,
            ApiError::AuthNotStarted => StatusCode::BAD_REQUEST,
            ApiError::PovCheckFailed => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }
}
