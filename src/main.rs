#![feature(proc_macro_hygiene, decl_macro, result_flattening)]
extern crate rocket;
use rocket::config::{Config, Environment};
use rocket::routes;

mod auth;
pub mod bigquery;
mod crawl;
mod map_json;
mod scan;
mod status;
mod up;
mod util;

/*
Code Summary:
The purpose of this module is to handle incoming HTTP requests and forward them to the appropriate sub-module. An API key is used for basic protection of the system, and requests without the correct key are rejected with "Not Authorized".

API Key:
The API key is set as the "api_key" Docker environment variable. The user must provide the correct key value in the "x-auth" field of the header in order to access the system.

Function:
- `get_env`: retrieves the value of a given environment variable or returns an error if the variable is missing.
- `main`: configures and launches the Rocket application, mounting the routes for each sub-module.
*/

pub fn get_env(name: &'static str) -> Result<String, String> {
    std::env::var(name).map_err(|_| format!("missing environment variable: {}", name))
}

fn main() {
    // Configure Rocket with the specified environment settings
    let config = Config::build(Environment::Development)
        .address("0.0.0.0") // Listen on all network interfaces
        .port(8000) // Listen on port 8000
        .finalize()
        .unwrap();

    // Mount the routes for each module
    rocket::custom(config)
        .mount("/", routes![up::catch_up])
        .mount("/", routes![scan::catch_scan])
        .mount("/", routes![crawl::catch_crawl])
        .mount("/", routes![status::catch_ready])
        .mount("/", routes![status::catch_health])
        .launch();
}
