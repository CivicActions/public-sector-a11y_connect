use crate::get_env;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;

/*
Code Summary:
This module handles the authorization of requests by verifying that the API key in the "x-auth" field of the header is valid. If the key is missing, invalid, or the environment variable containing the key is not set, the request is rejected with an appropriate status code. It defines a custom request guard, ApiKey, which checks if the x-auth header of an incoming request matches a given API key in the environment. The from_request method of ApiKey checks the header and environment variables, then returns an Outcome that is either a ApiKey or an ApiKeyError.


Variables:
    keys:
        A Vec of headers in the x-auth field of an incoming request.

    key:
        An Option that retrieves the value of the API_KEY environment variable.

    ApiKeyError:
        An enum type that specifies the possible errors that can occur during the request guard check.

    ApiKey:
         A struct that represents a valid API key.

Functions:
    from_request:
        This method is called by Rocket when it receives an incoming request. It checks the x-auth header and compares it to the API key in the environment. It then returns an Outcome that is either a ApiKey or an ApiKeyError.



Docker Vars:



Output:


Errors:
    BadCount:
        Returned when there are multiple API keys in the header.

    Missing:
        Returned when there is no API key in the header.

    Invalid:
        Returned when the API key in the header does not match the key in the environment.

    NotSet:
        Returned when the API_KEY environment variable is not set.


*/

#[derive(Debug)]
pub enum ApiKeyError {
    BadCount, // too many keys
    Missing,  // no keys
    Invalid,  // invalid key
    NotSet,   // environment variable not set
}

pub struct ApiKey(String);

impl<'a, 'r> FromRequest<'a, 'r> for ApiKey {
    type Error = ApiKeyError;

    // Check the x-auth header and compare to the API key in the environment
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("x-auth").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::Unauthorized, ApiKeyError::Missing)), // No API key found in header
            1 => match get_env("API_KEY") {
                Ok(key) if keys[0] == key => Outcome::Success(ApiKey(keys[0].to_string())), // Valid API key
                Ok(_) => Outcome::Failure((Status::Unauthorized, ApiKeyError::Invalid)), // Invalid API key
                Err(_) => Outcome::Failure((Status::InternalServerError, ApiKeyError::NotSet)), // Environment variable not set
            },
            _ => Outcome::Failure((Status::BadRequest, ApiKeyError::BadCount)), // Multiple API keys found in header
        }
    }
}
