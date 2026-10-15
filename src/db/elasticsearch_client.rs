//! Elasticsearch driver for dbuni, backed by the REST API via `reqwest`.
use crate::db::{
    DBUniBoxedConnection, DBUniConnectOptions, DBUniConnection, DBUniDriverFactory, DBUniParam,
    DBUniRawRow, missing_option, new_driver_runtime,
};
use crate::types::{
    DBUniDatabase, DBUniError, DBUniPayload, DBUniResult, DBUniResultT, DBUniValue,
};
/// An Elasticsearch connection backed by a reqwest client and a private runtime.
pub(crate) struct DBUniElasticsearchConnection {
    client: reqwest::Client,
    runtime: tokio::runtime::Runtime,
    base_url: String,
    user: String,
    password: String,
    index: String,
}
impl DBUniElasticsearchConnection {
    /// Builds the Elasticsearch base URL.
    fn build_base_url(options: &DBUniConnectOptions, port: u16) -> String {
        let scheme = options.get_option("scheme").unwrap_or("http");
        format!("{}://{}:{}", scheme, options.host, port)
    }
    /// Converts a serde_json value into a `DBUniValue`.
    fn json_to_uni(value: &serde_json::Value) -> DBUniValue {
        match value {
            serde_json::Value::Null => DBUniValue::Null,
            serde_json::Value::Bool(v) => DBUniValue::Bool(*v),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    DBUniValue::Int(i)
                } else if let Some(u) = n.as_u64() {
                    DBUniValue::UInt(u)
                } else if let Some(f) = n.as_f64() {
                    DBUniValue::Float(f)
                } else {
                    DBUniValue::Null
                }
            }
            serde_json::Value::String(s) => DBUniValue::Text(s.clone()),
            serde_json::Value::Array(items) => {
                DBUniValue::Array(items.iter().map(Self::json_to_uni).collect())
            }
            serde_json::Value::Object(obj) => DBUniValue::Map(
                obj.iter()
                    .map(|(k, v)| (DBUniValue::Text(k.clone()), Self::json_to_uni(v)))
                    .collect(),
            ),
        }
    }
    /// Parses the statement into a JSON document.
    fn parse_statement(statement: &str) -> DBUniResultT<serde_json::Value> {
        serde_json::from_str(statement.trim()).map_err(|e| DBUniError::SyntaxError {
            statement: statement.to_string(),
            message: format!("invalid Elasticsearch statement JSON: {e}"),
        })
    }
    /// Extracts a string field from the statement, or returns an error.
    fn required_str(doc: &serde_json::Value, key: &str) -> DBUniResultT<String> {
        doc.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| DBUniError::ParameterError(format!("missing required field: {key}")))
    }
    /// Extracts an optional string field.
    fn optional_str(doc: &serde_json::Value, key: &str) -> Option<String> {
        doc.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    }
    /// Extracts an optional JSON object field.
    fn optional_object(doc: &serde_json::Value, key: &str) -> Option<serde_json::Value> {
        doc.get(key).cloned()
    }
    /// Parses the first bound parameter as a JSON document.
    fn doc_from_params(params: &[DBUniParam], key: &str) -> DBUniResultT<serde_json::Value> {
        let Some(param) = params.first() else {
            return Err(missing_option(key));
        };
        let text = match param {
            DBUniParam::Json(text) | DBUniParam::Text(text) => text,
            _ => {
                return Err(DBUniError::ParameterError(format!(
                    "{key} must be a JSON document"
                )));
            }
        };
        serde_json::from_str(text)
            .map_err(|e| DBUniError::ParameterError(format!("invalid JSON for {key}: {e}")))
    }
    /// Sends a GET request to a path and returns the parsed JSON body.
    fn send_get(&self, path: &str) -> DBUniResultT<serde_json::Value> {
        let url = format!("{}{}", self.base_url, path);
        let client = self.client.clone();
        let user = self.user.clone();
        let password = self.password.clone();
        self.runtime.block_on(async move {
            let mut req = client.get(&url);
            if !user.is_empty() {
                req = req.basic_auth(user, Some(password));
            }
            let resp = req.send().await.map_err(|e| DBUniError::Driver {
                database: DBUniDatabase::Elasticsearch,
                code: None,
                message: e.to_string(),
            })?;
            let status = resp.status();
            let text = resp.text().await.map_err(|e| DBUniError::Driver {
                database: DBUniDatabase::Elasticsearch,
                code: None,
                message: e.to_string(),
            })?;
            if !status.is_success() {
                return Err(DBUniError::Driver {
                    database: DBUniDatabase::Elasticsearch,
                    code: Some(status.as_u16().to_string()),
                    message: text,
                });
            }
            serde_json::from_str(&text).map_err(|e| DBUniError::Serialization(e.to_string()))
        })
    }
    /// Sends a POST request with a JSON body and returns the parsed JSON body.
    fn send_post(&self, path: &str, body: &serde_json::Value) -> DBUniResultT<serde_json::Value> {
        let url = format!("{}{}", self.base_url, path);
        let client = self.client.clone();
        let user = self.user.clone();
        let password = self.password.clone();
        let body = body.clone();
        self.runtime.block_on(async move {
            let mut req = client.post(&url).json(&body);
            if !user.is_empty() {
                req = req.basic_auth(user, Some(password));
            }
            let resp = req.send().await.map_err(|e| DBUniError::Driver {
                database: DBUniDatabase::Elasticsearch,
                code: None,
                message: e.to_string(),
            })?;
            let status = resp.status();
            let text = resp.text().await.map_err(|e| DBUniError::Driver {
                database: DBUniDatabase::Elasticsearch,
                code: None,
                message: e.to_string(),
            })?;
            if !status.is_success() {
                return Err(DBUniError::Driver {
                    database: DBUniDatabase::Elasticsearch,
                    code: Some(status.as_u16().to_string()),
                    message: text,
                });
            }
            serde_json::from_str(&text).map_err(|e| DBUniError::Serialization(e.to_string()))
        })
    }
    /// Sends a PUT request with a JSON body and returns the parsed JSON body.
    fn send_put(&self, path: &str, body: &serde_json::Value) -> DBUniResultT<serde_json::Value> {
        let url = format!("{}{}", self.base_url, path);
        let client = self.client.clone();
        let user = self.user.clone();
        let password = self.password.clone();
        let body = body.clone();
        self.runtime.block_on(async move {
            let mut req = client.put(&url).json(&body);
            if !user.is_empty() {
                req = req.basic_auth(user, Some(password));
            }
            let resp = req.send().await.map_err(|e| DBUniError::Driver {
                database: DBUniDatabase::Elasticsearch,
                code: None,
                message: e.to_string(),
            })?;
            let status = resp.status();
            let text = resp.text().await.map_err(|e| DBUniError::Driver {
                database: DBUniDatabase::Elasticsearch,
                code: None,
                message: e.to_string(),
            })?;
            if !status.is_success() {
                return Err(DBUniError::Driver {
                    database: DBUniDatabase::Elasticsearch,
                    code: Some(status.as_u16().to_string()),
                    message: text,
                });
            }
            serde_json::from_str(&text).map_err(|e| DBUniError::Serialization(e.to_string()))
        })
    }
    /// Sends a DELETE request and returns the parsed JSON body.
    fn send_delete(&self, path: &str) -> DBUniResultT<serde_json::Value> {
        let url = format!("{}{}", self.base_url, path);
        let client = self.client.clone();
        let user = self.user.clone();
        let password = self.password.clone();
        self.runtime.block_on(async move {
            let mut req = client.delete(&url);
            if !user.is_empty() {
                req = req.basic_auth(user, Some(password));
            }
            let resp = req.send().await.map_err(|e| DBUniError::Driver {
                database: DBUniDatabase::Elasticsearch,
                code: None,
                message: e.to_string(),
            })?;
            let status = resp.status();
            let text = resp.text().await.map_err(|e| DBUniError::Driver {
                database: DBUniDatabase::Elasticsearch,
                code: None,
                message: e.to_string(),
            })?;
            if !status.is_success() {
                return Err(DBUniError::Driver {
                    database: DBUniDatabase::Elasticsearch,
                    code: Some(status.as_u16().to_string()),
                    message: text,
                });
            }
            serde_json::from_str(&text).map_err(|e| DBUniError::Serialization(e.to_string()))
        })
    }
    /// Resolves the default index name from options.
    fn resolve_index(options: &DBUniConnectOptions) -> String {
        if options.database.trim().is_empty() {
            "_all".to_string()
        } else {
            options.database.clone()
        }
    }
}
impl DBUniConnection for DBUniElasticsearchConnection {
    fn database(&self) -> DBUniDatabase {
        DBUniDatabase::Elasticsearch
    }
    fn query_raw(&mut self, statement: &str, params: &[DBUniParam]) -> DBUniResultT<DBUniResult> {
        let start = std::time::Instant::now();
        let spec = Self::parse_statement(statement)?;
        let op = spec.get("op").and_then(|v| v.as_str()).unwrap_or("search");
        let index = Self::optional_str(&spec, "index").unwrap_or_else(|| self.index.clone());
        let _ = params;
        let result = match op {
            "search" => {
                let body = Self::optional_object(&spec, "query")
                    .unwrap_or_else(|| serde_json::json!({ "query": { "match_all": {} } }));
                let path = format!("/{}/_search", index);
                self.send_post(&path, &body)?
            }
            "get" => {
                let id = Self::required_str(&spec, "id")?;
                let path = format!("/{}/_doc/{}", index, id);
                self.send_get(&path)?
            }
            "count" => {
                let body = Self::optional_object(&spec, "query")
                    .unwrap_or_else(|| serde_json::json!({ "query": { "match_all": {} } }));
                let path = format!("/{}/_count", index);
                self.send_post(&path, &body)?
            }
            "cat_indices" => {
                let path = "/_cat/indices?format=json".to_string();
                self.send_get(&path)?
            }
            other => {
                return Err(DBUniError::Unsupported(format!(
                    "unsupported Elasticsearch query op: {other}"
                )));
            }
        };
        let elapsed = start.elapsed().as_millis() as u64;
        // Extract hits for search responses, otherwise wrap the whole body.
        let documents: Vec<DBUniValue> =
            if let Some(hits) = result.get("hits").and_then(|h| h.get("hits")) {
                match hits.as_array() {
                    Some(items) => items
                        .iter()
                        .map(|item| Self::json_to_uni(item.get("_source").unwrap_or(item)))
                        .collect(),
                    None => Vec::new(),
                }
            } else if let Some(items) = result.as_array() {
                items.iter().map(Self::json_to_uni).collect()
            } else {
                vec![Self::json_to_uni(&result)]
            };
        let affected = Some(documents.len() as u64);
        Ok(DBUniResult::new(
            DBUniDatabase::Elasticsearch,
            elapsed,
            affected,
            DBUniPayload::Document(documents),
        ))
    }
    fn query_rows(
        &mut self,
        statement: &str,
        params: &[DBUniParam],
    ) -> DBUniResultT<Vec<DBUniRawRow>> {
        let spec = Self::parse_statement(statement)?;
        let op = spec.get("op").and_then(|v| v.as_str()).unwrap_or("search");
        let index = Self::optional_str(&spec, "index").unwrap_or_else(|| self.index.clone());
        let _ = params;
        let result = match op {
            "search" => {
                let body = Self::optional_object(&spec, "query")
                    .unwrap_or_else(|| serde_json::json!({ "query": { "match_all": {} } }));
                let path = format!("/{}/_search", index);
                self.send_post(&path, &body)?
            }
            "get" => {
                let id = Self::required_str(&spec, "id")?;
                let path = format!("/{}/_doc/{}", index, id);
                self.send_get(&path)?
            }
            "count" => {
                let body = Self::optional_object(&spec, "query")
                    .unwrap_or_else(|| serde_json::json!({ "query": { "match_all": {} } }));
                let path = format!("/{}/_count", index);
                self.send_post(&path, &body)?
            }
            "cat_indices" => {
                let path = "/_cat/indices?format=json".to_string();
                self.send_get(&path)?
            }
            other => {
                return Err(DBUniError::Unsupported(format!(
                    "unsupported Elasticsearch query op: {other}"
                )));
            }
        };
        let mut rows = Vec::new();
        if let Some(hits) = result.get("hits").and_then(|h| h.get("hits")) {
            if let Some(items) = hits.as_array() {
                for item in items {
                    let source = item.get("_source").unwrap_or(item);
                    if let Some(obj) = source.as_object() {
                        let columns: Vec<String> = obj.keys().cloned().collect();
                        let values: Vec<DBUniValue> = obj.values().map(Self::json_to_uni).collect();
                        rows.push(DBUniRawRow::new(columns, values));
                    }
                }
            }
        } else if let Some(items) = result.as_array() {
            for item in items {
                if let Some(obj) = item.as_object() {
                    let columns: Vec<String> = obj.keys().cloned().collect();
                    let values: Vec<DBUniValue> = obj.values().map(Self::json_to_uni).collect();
                    rows.push(DBUniRawRow::new(columns, values));
                }
            }
        } else if let Some(obj) = result.as_object() {
            let columns: Vec<String> = obj.keys().cloned().collect();
            let values: Vec<DBUniValue> = obj.values().map(Self::json_to_uni).collect();
            rows.push(DBUniRawRow::new(columns, values));
        }
        Ok(rows)
    }
    fn execute(&mut self, statement: &str, params: &[DBUniParam]) -> DBUniResultT<u64> {
        let spec = Self::parse_statement(statement)?;
        let op = spec.get("op").and_then(|v| v.as_str()).unwrap_or("index");
        let index = Self::optional_str(&spec, "index").unwrap_or_else(|| self.index.clone());
        match op {
            "index" => {
                let document = Self::doc_from_params(params, "document")?;
                let path = if let Some(id) = Self::optional_str(&spec, "id") {
                    format!("/{}/_doc/{}", index, id)
                } else {
                    format!("/{}/_doc", index)
                };
                let _ = self.send_post(&path, &document)?;
                Ok(1)
            }
            "update" => {
                let id = Self::required_str(&spec, "id")?;
                let update = Self::doc_from_params(params, "update")?;
                let body = serde_json::json!({ "doc": update });
                let path = format!("/{}/_update/{}", index, id);
                let _ = self.send_post(&path, &body)?;
                Ok(1)
            }
            "delete" => {
                let id = Self::required_str(&spec, "id")?;
                let path = format!("/{}/_doc/{}", index, id);
                let _ = self.send_delete(&path)?;
                Ok(1)
            }
            "create_index" => {
                let body = serde_json::json!({});
                let path = format!("/{}", index);
                let _ = self.send_put(&path, &body)?;
                Ok(1)
            }
            "delete_index" => {
                let path = format!("/{}", index);
                let _ = self.send_delete(&path)?;
                Ok(1)
            }
            other => Err(DBUniError::Unsupported(format!(
                "unsupported Elasticsearch execute op: {other}"
            ))),
        }
    }
    fn close(&mut self) -> DBUniResultT<()> {
        // The reqwest client closes automatically when dropped.
        Ok(())
    }
}
/// Factory for the Elasticsearch driver.
pub(crate) struct DBUniElasticsearchFactory;
impl DBUniDriverFactory for DBUniElasticsearchFactory {
    fn database(&self) -> DBUniDatabase {
        DBUniDatabase::Elasticsearch
    }
    fn connect(&self, options: &DBUniConnectOptions) -> DBUniResultT<DBUniBoxedConnection> {
        if options.host.trim().is_empty() {
            return Err(missing_option("host"));
        }
        if options.port == 0 {
            return Err(missing_option("port"));
        }
        let base_url = DBUniElasticsearchConnection::build_base_url(options, options.port);
        let runtime =
            new_driver_runtime().map_err(|e| DBUniError::ConnectionFailed(format!("{e}")))?;
        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| DBUniError::ConnectionFailed(format!("{e}")))?;
        Ok(Box::new(DBUniElasticsearchConnection {
            client,
            runtime,
            base_url,
            user: options.user.clone(),
            password: options.password.clone(),
            index: DBUniElasticsearchConnection::resolve_index(options),
        }))
    }
}
