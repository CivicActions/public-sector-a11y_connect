# API Documentation

Overview

## Scan

Body

```
"url": data.url,
"pageInsights": data.page_insights,
```

    // apply json mappings and upload to google big query

let mapper_bq_issues =
JsonMapper::new(serde_json::from_str(include_str!("../mapping/bq_issues.json")).unwrap());
let mapper_bq =
JsonMapper::new(serde_json::from_str(include_str!("../mapping/bq_crawls.json")).unwrap());
let mapper =
JsonMapper::new(serde_json::from_str(include_str!("../mapping/crawls.json")).unwrap());

## Crawl

et mut datapoints = Vec::new();
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
