use actix_web::{
    error::{self},
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum ApiError {
    #[display(fmt = "auth not started")]
    AuthNotStarted,

    #[display(fmt = "user not auth")]
    UserNotAuth,

    #[display(fmt = "invalid password")]
    InvalidHash,

    #[display(fmt = "user not exist")]
    UserNotExist,

    #[display(fmt = "json convertion failed")]
    JsonConvertionFailed,

    #[display(fmt = "internal state unavalible")]
    InternalStateUnavailable,

    #[display(fmt = "pov calc failled")]
    PovCheckFailed,
}

impl error::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

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
