use reqwest::Client;
use rocket::{http::Status, post, response::status, Data};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value as JsonValue;
use std::io::{BufReader, Read};
use std::time::Duration;

use crate::auth::ApiKey;
use crate::bigquery::store as bq_store;
use crate::get_env;
use crate::map_json::JsonMapper;
use crate::util::check_for_error;

/*

Code Summary:
    This code is a Rust Rocket web service endpoint for scanning a web page for accessibility issues. It sends a request to an accessibility testing service with a JSON payload and receives a JSON response, then applies some JSON mapping logic and stores the results in Google BigQuery.

Variables:
    ScanData:
        A struct for holding the JSON data received from the client's request. It includes two fields, url and page_insights.

    client:
        A reqwest::Client used to send the request to the accessibility testing service.

    json_data:
        A JSON object holding the payload to send with the request.

    mapper_bq_issues, mapper_bq, mapper:
        JsonMapper objects that hold mappings used to apply the JSON data to three different BigQuery tables.

Functions:
    catch_scan:
        A Rocket endpoint function that handles a POST request to the "/scan" path. It reads the JSON payload from the request and sends it to the accessibility testing service using client. Then it applies the JSON mappings to the result, stores the results in BigQuery, and returns the JSON data back to the client.

Output
        The function returns a JSON payload containing the results of the scan.

Error Messages
        The function may return a variety of status codes and error messages depending on the stage of the process where an error occurs. For example, if the input data is not properly formatted, the function returns a status code 400 Bad Request with a message indicating the parsing error. If the request to the accessibility testing service fails, the function returns a status code 500 Internal Server Error with a message indicating the reason for the failure. If mapping the JSON data to the BigQuery tables fails, the function returns a status code 500 Internal Server Error with a message indicating the reason for the failure.



*/

// Struct for holding the json body data
#[derive(Serialize, Deserialize, Debug)]
struct ScanData {
    url: String,
    page_insights: bool,
}

#[post("/scan", data = "<raw_data>")]
pub fn catch_scan(
    raw_data: Data,
    _key: ApiKey,
) -> Result<
    rocket::response::content::Json<String>,
    rocket::response::status::Custom<std::string::String>,
> {
    // Parse incoming request body into a ScanData struct
    let data: ScanData = serde_json::from_reader(BufReader::new(raw_data.open().take(1024 * 1024)))
        .map_err(|e| {
            status::Custom(
                Status::BadRequest,
                format!("failed to parse body data: {}", e),
            )
        })?;

    // Create a new reqwest Client
    let client = Client::new();

    // Creating the json data for the request
    let json_data = json!({
        "url": data.url,
        "pageInsights": data.page_insights,
    });

    // Create a new tokio Runtime to handle the async calls
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    // Send the request and parse the response
    let future = rt
        .block_on(
            client
                .post(format!(
                    "{}/scan",
                    // Get the A11Y_URL environment variable and append "/scan" to the end
                    get_env("A11Y_URL")
                        .map_err(|e| { status::Custom(Status::InternalServerError, e,) })?
                ))
                .header("Accept", "application/json")
                .header("Content-Type", "application/json")
                .header(
                    // Get the A11Y_JWT environment variable and set as Authorization header
                    "Authorization",
                    get_env("A11Y_JWT")
                        .map_err(|e| status::Custom(Status::InternalServerError, e))?,
                )
                .body(json_data.to_string())
                .timeout(Duration::from_secs(600)) // Wait up to 15 minutes
                .send(),
        )
        .map_err(|e| {
            status::Custom(
                Status::InternalServerError,
                format!("Error sending request: {}", e),
            )
        })?
        .json::<JsonValue>();

    // Parse response as JSON
    let response = rt.block_on(future).map_err(|e| {
        status::Custom(
            Status::InternalServerError,
            format!("Error parsing response: {}", e),
        )
    })?;

    // Check for any error in the response
    check_for_error(&response)?;

    // Create JSON mappers to map the response to the appropriate JSON structure for storage in BigQuery
    let mapper_bq_issues =
        JsonMapper::new(serde_json::from_str(include_str!("../mapping/bq_issues.json")).unwrap());
    let mapper_bq =
        JsonMapper::new(serde_json::from_str(include_str!("../mapping/bq_crawls.json")).unwrap());
    let mapper =
        JsonMapper::new(serde_json::from_str(include_str!("../mapping/crawls.json")).unwrap());

    // Map the response to the JSON structure for BigQuery and store in BigQuery
    let result_bq_issues = mapper_bq_issues.map(&response).map_err(|e| {
        status::Custom(
            Status::InternalServerError,
            format!("Error: The scan module failed to map json data: {:?}", e),
        )
    })?;
    rt.block_on(bq_store(
        "rusty_a11y".to_owned(),
        "issues".to_owned(),
        &result_bq_issues,
    ))
    .map_err(|e| {
        status::Custom(
            Status::InternalServerError,
            format!(
                "Error: The scan module failed to store data in google big query: {}",
                e
            ),
        )
    })?;

    // Map the response to the JSON structure for BigQuery and store in BigQuery for crawls
    let result_bq = mapper_bq.map(&response).map_err(|e| {
        status::Custom(
            Status::InternalServerError,
            format!("Error: The scan module failed to map json data: {:?}", e),
        )
    })?;
    rt.block_on(bq_store(
        "rusty_a11y".to_owned(),
        "crawls".to_owned(),
        &result_bq,
    ))
    .map_err(|e| {
        status::Custom(
            Status::InternalServerError,
            format!(
                "Error: The scan module failed to store data in google big query: {}",
                e
            ),
        )
    })?;

    // Map the response to the JSON structure and return a JSON object
    let result = mapper.map(&response).map_err(|e| {
        status::Custom(
            Status::InternalServerError,
            format!("Error: The scan module failed to map json data: {:?}", e),
        )
    })?;

    Ok(rocket::response::content::Json(result.to_string()))
}
