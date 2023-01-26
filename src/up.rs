extern crate reqwest;
mod modname {
    pub(crate) extern crate rocket;
}

extern crate serde_json;
extern crate tokio;
use rocket;
pub(crate) use rocket_contrib::json::Json;
use serde_json::Value;

#[post("/up", data = "<data>")]
pub(crate) async fn catch_up(data: Json<Value>) -> Result<String, String> {
    let action = data
        .get("action")
        .unwrap_or_else(|| &Value::Null)
        .as_str()
        .unwrap_or("");
    let url = data
        .get("url")
        .unwrap_or_else(|| &Value::Null)
        .as_str()
        .unwrap_or("");
    if let "cycle" = action {
        Ok("hello world".to_string())
    } else {
        let client = reqwest::Client::new();
        match client.get(url).send().await {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    Ok(format!("url: {}, status: up", url))
                } else {
                    Ok(format!("url: {}, status: down", url))
                }
            }
            Err(e) => Err(format!("Unable to reach the url: {} error: {:?}", url, e)),
        }
    }
}
