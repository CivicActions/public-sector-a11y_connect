extern crate reqwest;
extern crate rocket;
extern crate serde_json;
extern crate tokio;

use crate::auth::ApiKey;
use crate::bigquery::read_up_targets;
use crate::bigquery::store as bq_store;
use rocket::post;
use rocket::{http::Status, response::status, Data};
use serde_json::Value as JsonValue;
use std::io::Read;

/*
Code Summary:
    This Rust code defines a single function catch_up to check the status of web pages. The function takes the input raw_data: Data and _key: ApiKey and returns a Result<String, rocket::response::status::Custom<std::string::String>> where the string is the parsed JSON data. This function parses the input JSON data to check the status of all targets in the list or the status of a specific target, using an HTTP client, and then stores this data in Google BigQuery.

Variables:
    raw_data :
        An input variable of type Data which is provided to the function catch_up.
    _key :
        An ApiKey variable which is not used in this function.
    buf :
        A mutable vector buffer to read the data into.
    data :
        A map of string to JSON value that is either an empty map or is parsed from the buffer.
    client :
        An instance of reqwest::Client.
    rt :
        An instance of tokio::runtime::Runtime.

Functions:
    catch_up :
        This function takes input raw_data: Data and _key: ApiKey and returns a Result<String, rocket::response::status::Custom<std::string::String>> where the string is the parsed JSON data. The function first creates an empty buffer to read the data into, then opens the raw data and reads it into the buffer. It then checks the status of all targets in the list or the status of a specific target using an HTTP client and stores this data in Google BigQuery.

Docker Vars:
    None

Output:
    The output of the catch_up function is a Result<String, rocket::response::status::Custom<std::string::String>> where the string is the parsed JSON data.

Errors:
    failed to read body data: {} :
        Indicates that the function failed to read the input data.

    failed to parse body data: {} :
        Indicates that the function failed to parse the input data.

    target must be a string :
        Indicates that the input target is not of the expected type string.

    failed to store data to google big query: {} :
        Indicates that the function failed to store the data in Google BigQuery.

    failed to read target list from google big query: {} :
        Indicates that the function failed to read the target list from Google BigQuery.



*/

// This function takes a Rocket Data object as input
// It also takes an ApiKey that is currently unused
// It returns a Result<String, Custom<String>> where the string is the parsed JSON data
#[post("/up", data = "<raw_data>")]
pub(crate) fn catch_up(
    raw_data: Data,
    _key: ApiKey,
) -> Result<String, rocket::response::status::Custom<std::string::String>> {
    // Create an empty buffer to read the data into
    let mut buf = Vec::new();

    // Open the raw data and read it into the buffer
    // We limit the amount of data read to 1 MB
    raw_data
        .open()
        .take(1024 * 1024)
        .read_to_end(&mut buf)
        .map_err(|e| {
            status::Custom(
                Status::BadRequest,
                format!("failed to read body data: {}", e),
            )
        })?;

    // If the buffer is empty, return an empty Map object
    // Otherwise, parse the buffer as JSON
    let data: serde_json::Map<String, JsonValue> = if buf.is_empty() {
        serde_json::Map::new()
    } else {
        serde_json::from_slice(&buf).map_err(|e| {
            status::Custom(
                Status::BadRequest,
                format!("failed to parse body data: {}", e),
            )
        })?
    };

    // Create a new client to make HTTP requests
    let client = reqwest::Client::new();

    // Create a new runtime to run async tasks
    let rt = tokio::runtime::Runtime::new().unwrap();
    if let Some(target) = data.get("target") {
        // If a specific target is given, check its status
        let target = target.as_str().ok_or_else(|| {
            status::Custom(Status::BadRequest, "target must be a string".to_owned())
        })?;
        let status = match rt.block_on(client.get(target).send()) {
            Ok(response)
                if response.status().as_u16() >= 200 && response.status().as_u16() <= 399 =>
            {
                true
            }
            _ => false,
        };

        // Store target and status in BigQuery
        let data = serde_json::json!({"target": target, "status": status});
        rt.block_on(bq_store("rusty_a11y".to_owned(), "ups".to_owned(), &data))
            .map_err(|e| {
                status::Custom(
                    Status::InternalServerError,
                    format!("failed to store data to google big query: {}", e),
                )
            })?;
        Ok(data.to_string())
    } else {
        // If no specific target is given, check the status of all targets in the list
        let targets = rt
            .block_on(read_up_targets("rusty_a11y".to_owned()))
            .map_err(|e| {
                status::Custom(
                    Status::InternalServerError,
                    format!("failed to read target list from google big query: {}", e),
                )
            })?;
        let mut bq_json = Vec::new();
        for target in targets.into_iter() {
            // Check the status of each target
            let status = match rt.block_on(client.get(target.clone()).send()) {
                Ok(response)
                    if response.status().as_u16() >= 200 && response.status().as_u16() <= 399 =>
                {
                    true
                }
                _ => false,
            };
            let data = serde_json::json!({"target": target, "status": status});
            bq_json.push(data);
        }
        let msg = format!("scanned {} target(s)", bq_json.len());

        // Store target and status list in BigQuery
        rt.block_on(bq_store(
            "rusty_a11y".to_owned(),
            "ups".to_owned(),
            &serde_json::Value::Array(bq_json),
        ))
        .map_err(|e| {
            status::Custom(
                Status::InternalServerError,
                format!("failed to store data to google big query: {}", e),
            )
        })?;
        Ok(msg)
    }
}
