extern crate reqwest;
extern crate rocket;
extern crate serde_json;
extern crate tokio;

use crate::auth::ApiKey;
use crate::bigquery::read_up_targets;
use crate::bigquery::store as bq_store;
use rocket::post;
use rocket::{http::Status, response::status};
use serde_json::Value;

/*
        This module's purpose is to determine if a website is available or not.

        User sends `POST` request to `/up` endpoint. If the body contains a value for `target`, then the application sends a `GET` request to the target. If it responds with success, such as a 200 code, then the test is a success. That is sent back to the requester in the following format in the body.
        ```
        {"target": "TestedURL", "status": "siteStatus" }
        ```

        If the body of the incoming request is empty, then we want to do the below.
        1) Reply with `Beginning Scan` in the body
        2) Authenticate with Google Big Query (see bigquery.rs)
        3) Perform SQL query for urls to test (see bigquery.rs)
        4) Repeat below until all values from SQL query are exhausted
            4a) Send GET request to url
            4b) If success or error, perform SQL Query. I can write Query. (see bigquery.rs)
                - We will update the `site_status` to either T or F, based on the result. We need the `site_status` variable available to a placeholder SQL query.
            4c) Begin at 4a until URLs exhausted




| Input | Description | Example | Required | Location |
| ---: | :---: | :---: | :---: | :---: |
| 'target' | URL or domain to check | https://example.com/ or example.com | F | Body |





| Returns | Description | Example |
| ---: | :---: | :---: |
| `target` | Tested URL | https://example.com |
| `status` | Status of target. `up` or `down` | `up` |



*/

#[post("/up", data = "<data>")]
pub(crate) fn catch_up(
    data: String,
    _key: ApiKey,
) -> Result<String, rocket::response::status::Custom<std::string::String>> {
    let data: serde_json::Map<String, Value> = serde_json::from_str(&data).map_err(|e| {
        status::Custom(
            Status::BadRequest,
            format!("failed to parse body data: {}", e),
        )
    })?;
    let client = reqwest::Client::new();
    let rt = tokio::runtime::Runtime::new().unwrap();
    if let Some(target) = data.get("target") {
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
