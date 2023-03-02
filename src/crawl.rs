use reqwest::Client;
use rocket::{http::Status, post, response::status, Data};
use serde::de::IntoDeserializer;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value as JsonValue;
use std::io::{BufReader, Read};
use std::time::Duration;

use crate::auth::ApiKey;
use crate::bigquery;
use crate::bigquery::store as bq_store;
use crate::get_env;
use crate::map_json::JsonMapper;
use crate::util::check_for_error;

/*
Code Summary

This Rust code defines a Rocket endpoint and supporting functions for running a web accessibility audit on a given website URL. The endpoint is designed to handle two different actions: scan, which runs the accessibility audit on a single URL, and cycle, which runs the audit on a list of URLs stored in a Google BigQuery table. When the audit is complete, the data is mapped to the appropriate format and stored in BigQuery.


Variables
    CrawlData struct: defines the fields of the JSON body data for a single audit URL.

    catch_crawl function: Rocket endpoint for the /crawl route. Handles the incoming HTTP request, checks the action, and calls run_crawl() with the given data.

    run_crawl function: performs the web crawl by sending an HTTP POST request to an external API and handling the response. The data from the response is then mapped to the appropriate format and stored in BigQuery.

Functions
    check_for_error: checks the response data for any error messages and throws an error if any are found.

    bq_store: stores the response data in a Google BigQuery table.

Docker Vars
    None

Output
    catch_crawl returns a JSON string with the response data from run_crawl.

    run_crawl returns a Result<JsonValue, rocket::response::status::Custom<std::string::String>> with the mapped response data or an error message.

Error Messages
    - If an error occurs parsing the request body data, catch_crawl returns a BadRequest error message.
    - If an error occurs sending the HTTP request or parsing the response data, run_crawl returns an InternalServerError error message.
    - If an error occurs applying the JSON mappings, run_crawl returns an InternalServerError error message.
    - If an error occurs storing the data in BigQuery, run_crawl returns an InternalServerError error message.
    - If the response data contains any error messages, check_for_error throws an error.

*/

// Struct for holding the json body data
#[derive(Serialize, Deserialize, Debug)]
pub struct CrawlData {
    pub url: String,
    pub subdomains: bool,
    pub tld: bool,
    pub page_insights: bool,
}

// The endpoint for the `catch_crawl` function is `/crawl` with the HTTP method POST.
// It expects a JSON body containing the crawl data.
#[post("/crawl", data = "<raw_data>")]
pub fn catch_crawl(
    raw_data: Data,
    _key: ApiKey,
) -> Result<
    rocket::response::content::Json<String>,
    rocket::response::status::Custom<std::string::String>,
> {
    // Parse the JSON data from the request body into a Map
    let data: serde_json::Map<String, JsonValue> = serde_json::from_reader(BufReader::new(
        raw_data.open().take(1024 * 1024),
    ))
    .map_err(|e| {
        status::Custom(
            Status::BadRequest,
            format!("Request Error Failed to parse body data: {}", e),
        )
    })?;
    let rt = tokio::runtime::Runtime::new().unwrap();
    match data.get("action").map(|v| v.as_str()).flatten() {
        Some("scan") => {
            // If the action is `scan`, deserialize the JSON data into a CrawlData struct
            let data = CrawlData::deserialize(JsonValue::Object(data).into_deserializer())
                .map_err(|e| {
                    status::Custom(
                        Status::BadRequest,
                        format!("Request Error Failed to parse body data: {}", e),
                    )
                })?;
            // Run the crawl and return the response as a JSON string
            Ok(rocket::response::content::Json(
                run_crawl(data)?.to_string(),
            ))
        }

        // Cycle through crawl_targets
        Some("cycle") => {
            // If the action is `cycle`, fetch all crawl targets from Google BigQuery,
            // run the crawl on each target, and return an array of responses as a JSON string
            let targets = rt
                .block_on(bigquery::read_crawl_targets("rusty_a11y".to_owned()))
                .map_err(|e| {
                    status::Custom(
                        Status::InternalServerError,
                        format!(
                            "Database Error Error fetching crawl targets from google big query: {}",
                            e
                        ),
                    )
                })?;
            let mut responses = Vec::new();
            for target in targets {
                responses.push(run_crawl(target)?);
            }
            Ok(rocket::response::content::Json(
                serde_json::Value::Array(responses).to_string(),
            ))
        }
        Some(_) | None => Err(status::Custom(
            Status::BadRequest,
            "Request Error Action not specified or invalid".to_owned(),
        )),
    }
}

// Sends the crawl data to an API and maps the response data before storing it in Google BigQuery
fn run_crawl(
    data: CrawlData,
) -> Result<JsonValue, rocket::response::status::Custom<std::string::String>> {
    // Create a new reqwest client for sending HTTP requests
    let client = Client::new();
    // Creating the json data for the request
    let json_data = json!({
        "url": data.url,
        "subdomains": data.subdomains,
        "tld": data.tld,
        "pageInsights": data.page_insights,
    });

    // Create a new tokio runtime so we can send an asynchronous request
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    // Send the request and parse the response
    let future = rt
        .block_on(
            client
                .post(format!(
                    "{}/crawl",
                    get_env("A11Y_URL")
                        .map_err(|e| { status::Custom(Status::InternalServerError, e,) })?
                ))
                .header("Accept", "application/json")
                .header("Content-Type", "application/json")
                .header(
                    "Authorization",
                    get_env("A11Y_JWT")
                        .map_err(|e| status::Custom(Status::InternalServerError, e))?,
                )
                .body(json_data.to_string())
                .timeout(Duration::from_secs(600)) // Wait up to 15 minutes
                .send(),
        )
        .map_err(|e| {
            // If there was an error sending the request, return an internal server error with the error message
            status::Custom(
                Status::InternalServerError,
                format!("Request Error Problem sending request: {}", e),
            )
        })?
        .json::<JsonValue>();

    // If there was an error parsing the response, return an internal server error with the error message
    let response = rt.block_on(future).map_err(|e| {
        status::Custom(
            Status::InternalServerError,
            format!("Response Error Problem parsing response: {}", e),
        )
    })?;

    // Check if there was an error in the response and return an error if there was
    check_for_error(&response)?;

    // Apply json mappings and upload to google big query
    // Load the mapping files for converting the JSON response to the format we need to store in BigQuery
    let mapper_bq_issues =
        JsonMapper::new(serde_json::from_str(include_str!("../mapping/bq_issues.json")).unwrap());
    let mapper_bq =
        JsonMapper::new(serde_json::from_str(include_str!("../mapping/bq_crawls.json")).unwrap());
    let mapper =
        JsonMapper::new(serde_json::from_str(include_str!("../mapping/crawls.json")).unwrap());

    // Apply the mappings to the response JSON to get the format we need for the BigQuery tables
    let result_bq_issues = mapper_bq_issues.map(&response).map_err(|e| {
        // If there was an error applying the mapping, return an internal server error with the error message
        status::Custom(
            Status::InternalServerError,
            format!("Mapping Error Failed to map json data: {:?}", e),
        )
    })?;

    // Store the data in the BigQuery table for issues
    rt.block_on(bq_store(
        "rusty_a11y".to_owned(), //BigQuery Dataset
        "issues".to_owned(),     // BigQuery Table
        &result_bq_issues,       // Data to be stored
    ))
    .map_err(|e| {
        status::Custom(
            Status::InternalServerError,
            format!("Database Error Failed to store data in Big Query: {}", e),
        )
    })?;

    let result_bq = mapper_bq.map(&response).map_err(|e| {
        status::Custom(
            Status::InternalServerError,
            format!("Mapping Error Failed to map json data: {:?}", e),
        )
    })?;

    // Store the data in the BigQuery table for crawls
    rt.block_on(bq_store(
        "rusty_a11y".to_owned(), // BigQuery Dataset
        "crawls".to_owned(),     // BigQuery Table
        &result_bq,              // Data to be stored
    ))
    .map_err(|e| {
        status::Custom(
            Status::InternalServerError,
            format!("Database Error Failed to store data in Big Query: {}", e),
        )
    })?;

    // Return the mapped response data
    Ok(mapper.map(&response).map_err(|e| {
        status::Custom(
            Status::InternalServerError,
            format!("Mapping Error Failed to map json data: {:?}", e),
        )
    })?)
}
