use crate::get_env;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;

#[derive(Debug)]
pub enum ApiKeyError {
    BadCount,
    Missing,
    Invalid,
    NotSet,
}

pub struct ApiKey(String);
impl<'a, 'r> FromRequest<'a, 'r> for ApiKey {
    type Error = ApiKeyError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("x-auth").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::Unauthorized, ApiKeyError::Missing)),
            1 => match get_env("API_KEY") {
                Ok(key) if keys[0] == key => Outcome::Success(ApiKey(keys[0].to_string())),
                Ok(_) => Outcome::Failure((Status::Unauthorized, ApiKeyError::Invalid)),
                Err(_) => Outcome::Failure((Status::InternalServerError, ApiKeyError::NotSet)),
            },
            _ => Outcome::Failure((Status::BadRequest, ApiKeyError::BadCount)),
        }
    }
}
