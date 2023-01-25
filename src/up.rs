use reqwest::Client;
use rocket::response::status::Custom;
use rocket::{post, State};

#[post("/")]
pub async fn up(bigquery_client: State<bigquery::BigqueryClient>) -> Custom<String> {
    // Connect to BigQuery
    bigquery_client.connect().await?;

    // Execute query
    let results = bigquery_client
        .query("SELECT name FROM ca-a11y.domains WHERE scan_active = true")
        .await?;

    // create a reqwest client
    let client = Client::new();

    // Iterate over the results and make a GET request to each name
    for result in results {
        let name = result.get("name").unwrap();
        let response = client.get(name).send().await?;

        // check if the response is successful
        if response.status().is_success() {
            println!("GET request to {} succeeded", name);
        } else {
            println!("GET request to {} failed", name);
        }
    }

    Custom(200, "Begin scanning".to_string())
}
