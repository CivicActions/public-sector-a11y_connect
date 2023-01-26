use reqwest::Client;
use reqwest::Response;
use rocket::{
    data::FromData,
    http::{ContentType, Status},
    post,
    response::status,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

// Struct for holding the json body data
#[derive(Serialize, Deserialize, Debug)]
struct CrawlData {
    url: String,
    subdomains: bool,
    tld: bool,
    page_insights: bool,
}

/// This function handles requests to the /crawl endpoint.
/// It takes in a target url and sends a post request to the a11ywatch-backend
/// with the appropriate json data. It then waits up to 10 minutes for a response,
/// and sends that response back to the original request.
///
/// # Arguments
/// * `url` - The target url for the crawl request
///
/// # Returns
/// A rocket `Response` object with the status and body of the a11ywatch-backend response
///

#[post("/crawl?<url>", data = "<data>")]
pub fn catch_crawl(url: String, data: CrawlData) -> Result<response<'static>, reqwest::Error> {
    /*
    Creating a client for making the request
    */
    let client = Client::new();

    // Creating the json data for the request
    let json_data = json!({
        "url": data.url,
        "subdomains": data.subdomains,
        "tld": data.tld,
        "pageInsights": data.page_insights,
    });

    // Send the request and parse the response
    let response = client
        .post("https://a11ywatch-backend.public-sector-a11y.app.civicactions.net/api/crawl")
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .body(request_body.to_string())
        .timeout(Duration::from_secs(600)) // Wait up to 10 minutes
        .send()
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                format!("Error sending request: {}", e),
            )
        })?
        .json::<CrawlResponse>()
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                format!("Error parsing response: {}", e),
            )
        })?;

    Ok(Json(response))
}
