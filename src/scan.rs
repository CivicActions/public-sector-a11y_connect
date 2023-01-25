use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use tokio::time::timeout;

#[derive(Serialize, Deserialize, Debug)]
struct MappingRule {
    from: String,
    to: String,
}

async fn scan(
    client: &Client,
    url: &str,
    page_insights: bool,
    mapping: &HashMap<String, String>,
) -> Result<Value, reqwest::Error> {
    //
    // Send the POST request
    //
    let json = client
        .post("https://a11ywatch-backend.public-sector-a11y.app.civicactions.net/api/scan")
        .json(&json!({ "url": url, "pageInsights": page_insights }))
        .send()
        .await?
        .json()
        .await?;
    //
    // Wait for up to 15 seconds for the response
    //
    let json = timeout(Duration::from_secs(15), json).await?;

    //
    // Apply the mapping rules to the response
    //
    let mut json = json.as_object_mut().unwrap();
    for (from, to) in mapping {
        let value = json.remove(from).unwrap();
        json.insert(to.to_string(), value);
    }
    Ok(json)
}

fn main() {
    // Load the mapping rules from json mapping file
    //  Where should this file live? 🤷‍♂️
    //
    let json_str = fs::read_to_string("mapping_rules.json").unwrap();
    let mapping_rules: Vec<MappingRule> = serde_json::from_str(&json_str).unwrap();
    let mapping = mapping_rules
        .into_iter()
        .map(|rule| (rule.from, rule.to))
        .collect::<HashMap<_, _>>();
    let client = reqwest::Client::new();
    let url = "https://civicactions.com";
    let page_insights = true;
    scan(&client, &url, &page_insights, &mapping);
}
