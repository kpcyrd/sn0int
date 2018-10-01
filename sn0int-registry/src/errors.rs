pub use failure::{Error, ResultExt};
pub type Result<T> = ::std::result::Result<T, Error>;

use rocket::Request;
use rocket::http::Status;
use rocket::response::{self, Responder};

#[derive(Debug)]
pub struct ApiError(Error);

#[derive(Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}

pub type ApiResult<T> = ::std::result::Result<T, ApiError>;

impl<'r> Responder<'r> for ApiError {
    fn respond_to(self, _: &Request) -> response::Result<'static> {
        error!("Error: {:?}", self.0);
        Err(Status::InternalServerError)
    }
}

impl From<Error> for ApiError {
    fn from(error: Error) -> ApiError {
        ApiError(error)
    }
}
