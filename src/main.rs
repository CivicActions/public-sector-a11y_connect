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
            Goal
                I want to send HTTP requests to this endpoint and then forward the request to the applicable module


               API Key
                    I want to provide basic protection for this system. The user will set api_key in the Docker envs to a string. That string is needed to access the system. If the key is not present, it replies with `Not Authorized`
                    Key in Header
                        `{"x-auth": "api_key"}`

                    Sample Key  `CGPk5x72BIwcaWVV7RWs`

                    Auth is handled before the request is forwarded to the module

            Incoming requests explained in detail on appropriate module



*/

pub fn get_env(name: &'static str) -> Result<String, String> {
    std::env::var(name).map_err(|_| format!("missing environment variable: {}", name))
}

fn main() {
    let config = Config::build(Environment::Development)
        .address("0.0.0.0")
        .port(8000)
        .finalize()
        .unwrap();

    rocket::custom(config)
        .mount("/", routes![up::catch_up])
        .mount("/", routes![scan::catch_scan])
        .mount("/", routes![crawl::catch_crawl])
        .mount("/", routes![status::catch_ready])
        .mount("/", routes![status::catch_health])
        .launch();
}
