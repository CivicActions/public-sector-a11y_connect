use reqwest::Client;
use rocket::{http::Status, post, response::status};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value as JsonValue;
use std::time::Duration;

use crate::auth::ApiKey;
use crate::bigquery;
use crate::bigquery::store as bq_store;
use crate::get_env;
use crate::map_json::JsonMapper;

// Struct for holding the json body data
#[derive(Serialize, Deserialize, Debug)]
pub struct CrawlData {
    pub url: String,
    pub subdomains: bool,
    pub tld: bool,
    pub page_insights: bool,
}

/*
This function handles requests to the /crawl endpoint.
It takes in a target url and sends a post request to the `ally_url` endpoint with the appropriate json data. It then waits up to 10 minutes for a response, and sends that response back to the original request and saves result to BigQuery after mapping fields.

User sends request to /crawl endpoint.

A11y API Docs - who we send crawl requests to. The url we send requests to needs to be set via a docker env.
https://a11ywatch.com/api-info

| Input | Description | Example | Required | Location |
| ---: | :---: | :---: | :---: | :---: |
| action | To `scan` or `cycle` sites | action=cycle OR action=scan | Yes | URL Param |
| 'target` | Domain | example.com | Only if `action=scan` | Body |
| `sub` | BOOL if to scan subdomains | sub=true | Yes | URL Param |
| `tld` | BOOL if to scan top level domains | tld=false | Yes | URL Param |
| `page_insights` | Bool if to use Page Insights | page_insights=true | URL Param |

**If action=scan**
If action is scan, then the target from the request body is used as the crawl value below.

        curl --location --request POST 'ally_url' --header 'Content-Type: application/json' -d '{
            "url": "target",
            "subdomains": true,
            "tld": true,
            "pageInsights": true
        }'

The results are mapped via the crawls.json file. mapping/crawls.json


url string
The url to crawl and gather reports.

subdomains boolean
Include subdomains that match domain.


sitemap boolean
Extend crawl with sitemap links.


tld boolean
Include all TLD extensions that match domain.


pageInsights boolean
Run with additional google lighthouse report. [Not required if configured]

If subdomains & tld is enabled, the crawls can take a WHILE. As an example of what type of data we are waiting for, I've attached the full crawl of va.gov. It is not a small file and the a11y api took  minutes to complete it. Some sites take over 20 minutes to complete. I want to cut off crawls after 15 minutes.




**If action=cycle**

BigQuery will run a SQL query and return all the variables above. Those values are passed through to the crawl. When the crawl completes, the response is mapped to crawls.json and forwarded to the requester.

Once forwarded, see the note below. All crawl results always get saved to BigQuery.



**After any crarl completes**
The results are always written to Google BigQuery. The crawl is recorded first. See the bq_crawls.json file for the fields we need mapped.

The issues are then added to BigQuery. One row per issue. See bq_issues.json for the mapping.



*/

#[post("/crawl", data = "<data_str>")]
pub fn catch_crawl(
    data_str: String,
    _key: ApiKey,
) -> Result<
    rocket::response::content::Json<String>,
    rocket::response::status::Custom<std::string::String>,
> {
    let data: serde_json::Map<String, JsonValue> =
        serde_json::from_str(&data_str).map_err(|e| {
            status::Custom(
                Status::BadRequest,
                format!("failed to parse body data: {}", e),
            )
        })?;
    let rt = tokio::runtime::Runtime::new().unwrap();
    match data.get("action").map(|v| v.as_str()).flatten() {
        Some("scan") => {
            let data: CrawlData = serde_json::from_str(&data_str).map_err(|e| {
                status::Custom(
                    Status::BadRequest,
                    format!("failed to parse body data: {}", e),
                )
            })?;
            Ok(rocket::response::content::Json(
                run_crawl(data)?.to_string(),
            ))
        }
        Some("cycle") => {
            let targets = rt
                .block_on(bigquery::read_crawl_targets("rusty_a11y".to_owned()))
                .map_err(|e| {
                    status::Custom(
                        Status::InternalServerError,
                        format!("Error fetching crawl targets from google big query: {}", e),
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
            "action not specified or invalid".to_owned(),
        )),
    }
}

fn run_crawl(
    data: CrawlData,
) -> Result<JsonValue, rocket::response::status::Custom<std::string::String>> {
    let client = Client::new();
    // Creating the json data for the request
    let json_data = json!({
        "url": data.url,
        "subdomains": data.subdomains,
        "tld": data.tld,
        "pageInsights": data.page_insights,
    });
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

    Ok(mapper.map(&response).map_err(|e| {
        status::Custom(
            Status::InternalServerError,
            format!("failed to map json data: {:?}", e),
        )
    })?)
}
