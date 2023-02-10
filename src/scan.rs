use reqwest::Client;
use rocket::{http::Status, post, response::status};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value as JsonValue;
use std::time::Duration;

use crate::auth::ApiKey;
use crate::bigquery::store as bq_store;
use crate::get_env;
use crate::map_json::JsonMapper;

// Struct for holding the json body data
#[derive(Serialize, Deserialize, Debug)]
struct ScanData {
    url: String,
    page_insights: bool,
}

#[post("/scan", data = "<data_str>")]
pub fn catch_scan(
    data_str: String,
    _key: ApiKey,
) -> Result<
    rocket::response::content::Json<String>,
    rocket::response::status::Custom<std::string::String>,
> {
    let client = Client::new();

    let data: ScanData = serde_json::from_str(&data_str).map_err(|e| {
        status::Custom(
            Status::BadRequest,
            format!("failed to parse body data: {}", e),
        )
    })?;

    // Creating the json data for the request
    let json_data = json!({
        "url": data.url,
        "pageInsights": data.page_insights,
    });

    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    // Send the request and parse the response
    let future = rt
        .block_on(
            client
                .post(format!(
                    "{}/scan",
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
            status::Custom(
                Status::InternalServerError,
                format!("Error sending request: {}", e),
            )
        })?
        .json::<JsonValue>();

    let response = rt.block_on(future).map_err(|e| {
        status::Custom(
            Status::InternalServerError,
            format!("Error parsing response: {}", e),
        )
    })?;

    // apply json mappings and upload to google big query
    let mapper_bq_issues =
        JsonMapper::new(serde_json::from_str(include_str!("../mapping/bq_issues.json")).unwrap());
    let mapper_bq =
        JsonMapper::new(serde_json::from_str(include_str!("../mapping/bq_crawls.json")).unwrap());
    let mapper =
        JsonMapper::new(serde_json::from_str(include_str!("../mapping/crawls.json")).unwrap());

    let result_bq_issues = mapper_bq_issues.map(&response).map_err(|e| {
        status::Custom(
            Status::InternalServerError,
            format!("failed to map json data: {:?}", e),
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
            format!("failed to store data to google big query: {}", e),
        )
    })?;

    let result_bq = mapper_bq.map(&response).map_err(|e| {
        status::Custom(
            Status::InternalServerError,
            format!("failed to map json data: {:?}", e),
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
            format!("failed to store data to google big query: {}", e),
        )
    })?;

    let result = mapper.map(&response).map_err(|e| {
        status::Custom(
            Status::InternalServerError,
            format!("failed to map json data: {:?}", e),
        )
    })?;

    Ok(rocket::response::content::Json(result.to_string()))
}
