use crate::crawl::CrawlData;
use crate::get_env;
use gcp_bigquery_client;
use sea_query;
use sea_query::types::Iden;
use serde_json::Value as JsonValue;
use std::boxed::Box;

use gcp_bigquery_client::Client;

/*
Code Summary:
This module contains functions to retrieve data from and store data in Google BigQuery. The following functions are defined:

read_up_targets: retrieves a list of URLs to be crawled from the up_targets table in the specified dataset
read_crawl_targets: retrieves a list of crawl targets from the crawl_targets table in the specified dataset
store: stores a JSON object in the specified table in the specified dataset in Google BigQuery


API Key:
The GOOGLE_APPLICATION_CREDENTIALS environment variable should be set to the path of a service account key file for a Google Cloud project.

The GOOGLE_PROJECT_ID environment variable should be set to the ID of the Google Cloud project.


// Implementation details:

The read_up_targets function executes a query to retrieve a list of URLs to be crawled from the up_targets table in the specified dataset. The dataset_name parameter is the name of the dataset containing the table. The function returns a vector of strings representing the URLs.

The read_crawl_targets function executes a query to retrieve a list of crawl targets from the crawl_targets table in the specified dataset. The dataset_name parameter is the name of the dataset containing the table. The function returns a vector of CrawlData structs representing the crawl targets.

The store function stores a JSON object in the specified table in the specified dataset in Google BigQuery. The dataset_name parameter is the name of the dataset containing the table, the table_name parameter is the name of the table to store the data in, and the object parameter is a reference to a JSON object to store. The function returns Ok(()) if the operation was successful, and an error message as a string if the operation failed.

The MyIden struct is an implementation of the Iden trait from the sea-query crate, which allows us to use custom identifiers when building SQL queries.

The sea-query crate is used to construct SQL queries.
*/

struct MyIden(String);
impl Iden for MyIden {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "{}", self.0).unwrap();
    }
}

// Retrieve a list of URLs to crawl
pub async fn read_up_targets(dataset_name: String) -> Result<Vec<String>, String> {
    // Create a client to communicate with Google BigQuery
    let client = Client::from_service_account_key_file(&get_env("GOOGLE_APPLICATION_CREDENTIALS")?)
        .await
        .map_err(|e| format!("{}", e))?;

    // Query the `up_targets` table to retrieve URLs
    let mut result_set = client
        .job()
        .query(
            &get_env("GOOGLE_PROJECT_ID")?,
            gcp_bigquery_client::model::query_request::QueryRequest::new(format!(
                "SELECT * FROM {}.up_targets",
                dataset_name
            )),
        )
        .await
        .map_err(|e| format!("{}", e))?;

    // Collect the URLs into a vector
    let mut urls = Vec::new();
    while result_set.next_row() {
        if let Some(url) = result_set
            .get_string_by_name("url")
            .map_err(|e| format!("{}", e))?
        {
            urls.push(url);
        }
    }
    Ok(urls)
}

// Retrieve a list of crawl targets
pub async fn read_crawl_targets(dataset_name: String) -> Result<Vec<CrawlData>, String> {
    // Create a client to communicate with Google BigQuery
    let client = Client::from_service_account_key_file(&get_env("GOOGLE_APPLICATION_CREDENTIALS")?)
        .await
        .map_err(|e| format!("{}", e))?;

    // Query the `crawl_targets` table to retrieve crawl targets
    let mut result_set = client
        .job()
        .query(
            &get_env("GOOGLE_PROJECT_ID")?,
            gcp_bigquery_client::model::query_request::QueryRequest::new(format!(
                "SELECT * FROM {}.crawl_targets",
                dataset_name
            )),
        )
        .await
        .map_err(|e| format!("{}", e))?;

    // Collect the crawl targets into a vector
    let mut datapoints = Vec::new();
    while result_set.next_row() {
        let url = result_set
            .get_string_by_name("url")
            .map_err(|e| format!("invalid data from google big query, error: {}", e))?;
        let subdomains = result_set
            .get_bool_by_name("subdomains")
            .map_err(|e| format!("invalid data from google big query, error: {}", e))?;
        let tld = result_set
            .get_bool_by_name("tld")
            .map_err(|e| format!("invalid data from google big query, error: {}", e))?;
        let page_insights = result_set
            .get_bool_by_name("page_insights")
            .map_err(|e| format!("invalid data from google big query, error: {}", e))?;

        if url.is_none() {
            continue;
        }
        if subdomains.is_none() {
            continue;
        }
        if tld.is_none() {
            continue;
        }
        if page_insights.is_none() {
            continue;
        }
        datapoints.push(CrawlData {
            url: url.unwrap(),
            subdomains: subdomains.unwrap(),
            tld: tld.unwrap(),
            page_insights: page_insights.unwrap(),
        })
    }
    Ok(datapoints)
}

// Store a JSON object in a BigQuery table
pub async fn store(
    dataset_name: String,
    table_name: String,
    object: &JsonValue,
) -> Result<(), String> {
    // Create a client to communicate with Google BigQuery
    let client = Client::from_service_account_key_file(&get_env("GOOGLE_APPLICATION_CREDENTIALS")?)
        .await
        .map_err(|e| format!("{}", e))?;

    // Define the table we want to store the data in
    let table = sea_query::types::TableRef::SchemaTable(
        sea_query::types::SeaRc::new(MyIden(dataset_name)),
        sea_query::types::SeaRc::new(MyIden(table_name)),
    );

    // Collect the names of the columns in the JSON object
    let mut columns_str = Vec::new();
    match object {
        JsonValue::Array(arr) => {
            for o in arr.iter() {
                for k in o
                    .as_object()
                    .ok_or_else(|| "expected object for bigquery::store".to_owned())?
                    .keys()
                {
                    if !columns_str.contains(k) {
                        columns_str.push(k.to_owned())
                    }
                }
            }
        }
        JsonValue::Object(obj) => {
            for k in obj.keys() {
                if !columns_str.contains(k) {
                    columns_str.push(k.to_owned())
                }
            }
        }
        _ => return Err("expected array or object for bigquery::store".to_owned()),
    }

    // Collect the values of the columns in the JSON object
    let mut values_json = Vec::new();
    match object {
        JsonValue::Object(obj) => {
            let mut cols = Vec::new();
            for col_name in columns_str.iter() {
                cols.push(obj.get(col_name));
            }
            values_json.push(cols);
        }
        JsonValue::Array(arr) => {
            for entry in arr.iter() {
                // unwrap: the mapping shouldn't return an array of non-objects
                let obj = entry.as_object().unwrap();
                let mut cols = Vec::new();
                for col_name in columns_str.iter() {
                    cols.push(obj.get(col_name));
                }
                values_json.push(cols);
            }
        }
        _ => return Err("expected array or object for bigquery::store".to_owned()),
    }

    // Convert the JSON values to SQL values
    let mut values = Vec::new();
    for json_entry_vec in values_json.into_iter() {
        let mut v = Vec::new();
        for json_entry in json_entry_vec {
            let sql_entry = match json_entry {
                Some(serde_json::Value::Null) => sea_query::value::Value::Bool(None),
                Some(serde_json::Value::Number(n)) => {
                    if let Some(ni64) = n.as_i64() {
                        sea_query::value::Value::BigInt(Some(ni64))
                    } else if let Some(nf64) = n.as_f64() {
                        sea_query::value::Value::Double(Some(nf64))
                    } else {
                        sea_query::value::Value::Bool(None)
                    }
                }
                Some(serde_json::Value::String(s)) => {
                    sea_query::value::Value::String(Some(Box::new(s.to_owned())))
                }
                Some(serde_json::Value::Bool(b)) => sea_query::value::Value::Bool(Some(*b)),
                _ => sea_query::value::Value::Bool(None),
            };
            v.push(sea_query::expr::SimpleExpr::Value(sql_entry));
        }
        if !v.is_empty() {
            values.push(v);
        }
    }

    // Define the columns in the table
    let columns = columns_str.iter().map(|c| MyIden(c.to_owned()));

    if !values.is_empty() {
        for chunk in values.chunks(64) {
            let mut query = sea_query::Query::insert();

            query.into_table(table.clone());
            query.columns(columns.clone());

            for set in chunk.into_iter() {
                query.values(set.into_iter().map(|i| i.clone())).unwrap();
            }

            let query = query.to_string(sea_query::backend::MysqlQueryBuilder);
            let query = gcp_bigquery_client::model::query_request::QueryRequest::new(query);
            client
                .job()
                .query(&get_env("GOOGLE_PROJECT_ID")?, query)
                .await
                .map_err(|e| format!("{}", e))?;
        }
    }
    Ok(())
}
