use crate::crawl::CrawlData;
use crate::get_env;
use gcp_bigquery_client;
use sea_query;
use sea_query::types::Iden;
use serde_json::Value as JsonValue;
use std::boxed::Box;

use gcp_bigquery_client::Client;

struct MyIden(String);
impl Iden for MyIden {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "{}", self.0).unwrap();
    }
}

pub async fn read_up_targets(dataset_name: String) -> Result<Vec<String>, String> {
    let client = Client::from_authorized_user_secret(&get_env("GOOGLE_CLOUD_KEY")?)
        .await
        .map_err(|e| format!("{}", e))?;
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

pub async fn read_crawl_targets(dataset_name: String) -> Result<Vec<CrawlData>, String> {
    let client = Client::from_authorized_user_secret(&get_env("GOOGLE_CLOUD_KEY")?)
        .await
        .map_err(|e| format!("{}", e))?;
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

pub async fn store(
    dataset_name: String,
    table_name: String,
    object: &JsonValue,
) -> Result<(), String> {
    let client = Client::from_authorized_user_secret(&get_env("GOOGLE_CLOUD_KEY")?)
        .await
        .map_err(|e| format!("{}", e))?;

    let table = sea_query::types::TableRef::SchemaTable(
        sea_query::types::SeaRc::new(MyIden(dataset_name)),
        sea_query::types::SeaRc::new(MyIden(table_name)),
    );

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
                _ => sea_query::value::Value::Bool(None),
            };
            v.push(sea_query::expr::SimpleExpr::Value(sql_entry));
        }
        if !v.is_empty() {
            values.push(v);
        }
    }

    let columns = columns_str.iter().map(|c| MyIden(c.to_owned()));

    if !values.is_empty() {
        let mut query = sea_query::Query::insert();

        query.into_table(table);
        query.columns(columns.clone());

        for set in values.into_iter() {
            query.values(set).unwrap();
        }

        let query = query.to_string(sea_query::backend::MysqlQueryBuilder);
        let query = gcp_bigquery_client::model::query_request::QueryRequest::new(query);
        client
            .job()
            .query(&get_env("GOOGLE_PROJECT_ID")?, query)
            .await
            .map_err(|e| format!("{}", e))?;
    }
    Ok(())
}
