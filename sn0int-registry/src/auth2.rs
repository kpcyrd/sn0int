use crate::errors::*;
use crate::models::AuthToken;
use crate::db::Connection;
use rocket::http::Status;
use rocket::{Request, Outcome};
use rocket::request::{self, FromRequest};
use crate::github::GithubAuthenticator;


pub struct AuthHeader(String);

impl AuthHeader {
    pub fn verify(&self, connection: &Connection) -> Result<String> {
        let session = AuthToken::read(&self.0, &connection)?;
        let client = GithubAuthenticator::from_env()?;
        client.get_username(&session.access_token)
            .map_err(Error::from)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for AuthHeader {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let headers = request.headers();
        match headers.get_one("Auth") {
            Some(session) => Outcome::Success(AuthHeader(session.to_string())),
            None => Outcome::Failure((Status::BadRequest, ())),
        }
    }
}
