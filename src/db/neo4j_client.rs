//! Neo4j driver for dbuni, backed by the HTTP transactional endpoint via `reqwest`.
use crate::db::{
    DBUniBoxedConnection, DBUniConnectOptions, DBUniConnection, DBUniDriverFactory, DBUniParam,
    DBUniRawRow, missing_option, new_driver_runtime,
};
use crate::types::{
    DBUniDatabase, DBUniError, DBUniGraphResult, DBUniPayload, DBUniResult, DBUniResultT,
    DBUniValue,
};
/// A Neo4j connection backed by a reqwest client and a private runtime.
pub(crate) struct DBUniNeo4jConnection {
    client: reqwest::Client,
    runtime: tokio::runtime::Runtime,
    base_url: String,
    user: String,
    password: String,
    database: String,
}
impl DBUniNeo4jConnection {
    /// Builds the Neo4j base URL.
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
    /// Renders a `DBUniParam` as a Neo4j Cypher parameter value.
    fn param_to_json(param: &DBUniParam) -> serde_json::Value {
        match param {
            DBUniParam::Null => serde_json::Value::Null,
            DBUniParam::Bool(v) => serde_json::Value::Bool(*v),
            DBUniParam::Int(v) => serde_json::json!(v),
            DBUniParam::UInt(v) => serde_json::json!(v),
            DBUniParam::Float(v) => serde_json::json!(v),
            DBUniParam::Text(v) => serde_json::Value::String(v.clone()),
            DBUniParam::Bytes(v) => {
                let hex: String = v.iter().map(|b| format!("{b:02x}")).collect();
                serde_json::Value::String(hex)
            }
            DBUniParam::Timestamp(v) => serde_json::json!(v),
            DBUniParam::Json(v) => {
                serde_json::from_str(v).unwrap_or_else(|_| serde_json::Value::String(v.clone()))
            }
        }
    }
    /// Builds the parameters object for the Neo4j transaction body.
    fn build_params(params: &[DBUniParam]) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        for (i, p) in params.iter().enumerate() {
            map.insert(format!("p{i}"), Self::param_to_json(p));
        }
        serde_json::Value::Object(map)
    }
    /// Converts a Neo4j node object into a `DBUniValue::Map`.
    fn node_to_uni(node: &serde_json::Value) -> DBUniValue {
        let mut map = Vec::new();
        if let Some(labels) = node.get("labels").and_then(|v| v.as_array()) {
            let label_values: Vec<DBUniValue> = labels
                .iter()
                .map(|l| DBUniValue::Text(l.as_str().unwrap_or("").to_string()))
                .collect();
            map.push((
                DBUniValue::Text("__labels".to_string()),
                DBUniValue::Array(label_values),
            ));
        }
        if let Some(props) = node.get("properties").and_then(|v| v.as_object()) {
            for (k, v) in props {
                map.push((DBUniValue::Text(k.clone()), Self::json_to_uni(v)));
            }
        }
        DBUniValue::Map(map)
    }
    /// Converts a Neo4j relationship object into a `DBUniValue::Map`.
    fn rel_to_uni(rel: &serde_json::Value) -> DBUniValue {
        let mut map = Vec::new();
        if let Some(t) = rel.get("type").and_then(|v| v.as_str()) {
            map.push((
                DBUniValue::Text("__type".to_string()),
                DBUniValue::Text(t.to_string()),
            ));
        }
        if let Some(props) = rel.get("properties").and_then(|v| v.as_object()) {
            for (k, v) in props {
                map.push((DBUniValue::Text(k.clone()), Self::json_to_uni(v)));
            }
        }
        DBUniValue::Map(map)
    }
    /// Converts a Neo4j path object into a `DBUniValue::Map`.
    fn path_to_uni(path: &serde_json::Value) -> DBUniValue {
        let mut map = Vec::new();
        if let Some(nodes) = path.get("nodes").and_then(|v| v.as_array()) {
            let node_values: Vec<DBUniValue> = nodes.iter().map(Self::node_to_uni).collect();
            map.push((
                DBUniValue::Text("nodes".to_string()),
                DBUniValue::Array(node_values),
            ));
        }
        if let Some(rels) = path.get("relationships").and_then(|v| v.as_array()) {
            let rel_values: Vec<DBUniValue> = rels.iter().map(Self::rel_to_uni).collect();
            map.push((
                DBUniValue::Text("relationships".to_string()),
                DBUniValue::Array(rel_values),
            ));
        }
        DBUniValue::Map(map)
    }
    /// Sends a Cypher statement to the Neo4j transactional HTTP endpoint.
    fn run_cypher(
        &self,
        statement: &str,
        params: &[DBUniParam],
    ) -> DBUniResultT<serde_json::Value> {
        let url = format!("{}/db/{}/tx/commit", self.base_url, self.database);
        let client = self.client.clone();
        let user = self.user.clone();
        let password = self.password.clone();
        let body = serde_json::json!({
            "statements": [{
                "statement": statement,
                "parameters": Self::build_params(params),
            }]
        });
        self.runtime.block_on(async move {
            let resp = client
                .post(&url)
                .basic_auth(user, Some(password))
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| DBUniError::Driver {
                    database: DBUniDatabase::Neo4j,
                    code: None,
                    message: e.to_string(),
                })?;
            let status = resp.status();
            let text = resp.text().await.map_err(|e| DBUniError::Driver {
                database: DBUniDatabase::Neo4j,
                code: None,
                message: e.to_string(),
            })?;
            if !status.is_success() {
                return Err(DBUniError::Driver {
                    database: DBUniDatabase::Neo4j,
                    code: Some(status.as_u16().to_string()),
                    message: text,
                });
            }
            serde_json::from_str(&text).map_err(|e| DBUniError::Serialization(e.to_string()))
        })
    }
    /// Extracts rows from the first result of a Neo4j response.
    fn extract_rows(
        response: &serde_json::Value,
    ) -> DBUniResultT<(Vec<String>, Vec<Vec<DBUniValue>>)> {
        if let Some(errors) = response.get("errors").and_then(|v| v.as_array()) {
            if !errors.is_empty() {
                let message = errors
                    .iter()
                    .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
                    .collect::<Vec<_>>()
                    .join("; ");
                return Err(DBUniError::Driver {
                    database: DBUniDatabase::Neo4j,
                    code: None,
                    message,
                });
            }
        }
        let results = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| DBUniError::Serialization("missing results array".to_string()))?;
        let first = match results.first() {
            Some(r) => r,
            None => return Ok((Vec::new(), Vec::new())),
        };
        let columns: Vec<String> = first
            .get("columns")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|c| c.as_str().unwrap_or("").to_string())
                    .collect()
            })
            .unwrap_or_default();
        let mut out_rows = Vec::new();
        if let Some(data) = first.get("data").and_then(|v| v.as_array()) {
            for item in data {
                if let Some(row) = item.get("row").and_then(|v| v.as_array()) {
                    let values: Vec<DBUniValue> = row.iter().map(Self::json_to_uni).collect();
                    out_rows.push(values);
                }
            }
        }
        Ok((columns, out_rows))
    }
    /// Resolves the target database name from options.
    fn resolve_database(options: &DBUniConnectOptions) -> String {
        if options.database.trim().is_empty() {
            "neo4j".to_string()
        } else {
            options.database.clone()
        }
    }
}
impl DBUniConnection for DBUniNeo4jConnection {
    fn database(&self) -> DBUniDatabase {
        DBUniDatabase::Neo4j
    }
    fn query_raw(&mut self, statement: &str, params: &[DBUniParam]) -> DBUniResultT<DBUniResult> {
        let start = std::time::Instant::now();
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            return Err(DBUniError::SyntaxError {
                statement: statement.to_string(),
                message: "empty statement".to_string(),
            });
        }
        let response = self.run_cypher(trimmed, params)?;
        let (columns, rows) = Self::extract_rows(&response)?;
        // Classify the payload: if the first column is a node/path map,
        // produce a graph result; otherwise fall back to an array of maps.
        let mut nodes: Vec<DBUniValue> = Vec::new();
        let mut edges: Vec<DBUniValue> = Vec::new();
        let mut paths: Vec<DBUniValue> = Vec::new();
        let mut generic: Vec<DBUniValue> = Vec::new();
        let mut has_graph = false;
        for row in &rows {
            for (index, value) in row.iter().enumerate() {
                let column = columns.get(index).map(|s| s.as_str()).unwrap_or("");
                if column.contains("path") {
                    paths.push(value.clone());
                    has_graph = true;
                } else if column.contains("rel") || column.contains("edge") {
                    edges.push(value.clone());
                    has_graph = true;
                } else if column.contains("node") {
                    nodes.push(value.clone());
                    has_graph = true;
                }
                generic.push(value.clone());
            }
        }
        let elapsed = start.elapsed().as_millis() as u64;
        if has_graph {
            let affected = Some((nodes.len() + edges.len() + paths.len()) as u64);
            Ok(DBUniResult::new(
                DBUniDatabase::Neo4j,
                elapsed,
                affected,
                DBUniPayload::Graph(DBUniGraphResult {
                    nodes,
                    edges,
                    paths,
                }),
            ))
        } else {
            let affected = Some(generic.len() as u64);
            Ok(DBUniResult::new(
                DBUniDatabase::Neo4j,
                elapsed,
                affected,
                DBUniPayload::Array(generic),
            ))
        }
    }
    fn query_rows(
        &mut self,
        statement: &str,
        params: &[DBUniParam],
    ) -> DBUniResultT<Vec<DBUniRawRow>> {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            return Err(DBUniError::SyntaxError {
                statement: statement.to_string(),
                message: "empty statement".to_string(),
            });
        }
        let response = self.run_cypher(trimmed, params)?;
        let (columns, rows) = Self::extract_rows(&response)?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(DBUniRawRow::new(columns.clone(), row));
        }
        Ok(out)
    }
    fn execute(&mut self, statement: &str, params: &[DBUniParam]) -> DBUniResultT<u64> {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            return Err(DBUniError::SyntaxError {
                statement: statement.to_string(),
                message: "empty statement".to_string(),
            });
        }
        let response = self.run_cypher(trimmed, params)?;
        if let Some(errors) = response.get("errors").and_then(|v| v.as_array()) {
            if !errors.is_empty() {
                let message = errors
                    .iter()
                    .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
                    .collect::<Vec<_>>()
                    .join("; ");
                return Err(DBUniError::Driver {
                    database: DBUniDatabase::Neo4j,
                    code: None,
                    message,
                });
            }
        }
        // Neo4j returns counters inside the stats object.
        if let Some(results) = response.get("results").and_then(|v| v.as_array()) {
            for result in results {
                if let Some(stats) = result.get("stats") {
                    let mut total: u64 = 0;
                    for key in [
                        "nodes_created",
                        "nodes_deleted",
                        "relationships_created",
                        "relationships_deleted",
                        "properties_set",
                        "labels_added",
                        "labels_removed",
                        "indexes_added",
                        "indexes_removed",
                        "constraints_added",
                        "constraints_removed",
                    ] {
                        if let Some(value) = stats.get(key).and_then(|v| v.as_u64()) {
                            total += value;
                        }
                    }
                    if total > 0 {
                        return Ok(total);
                    }
                }
            }
        }
        Ok(0)
    }
    fn close(&mut self) -> DBUniResultT<()> {
        // The reqwest client closes automatically when dropped.
        Ok(())
    }
}
/// Factory for the Neo4j driver.
pub(crate) struct DBUniNeo4jFactory;
impl DBUniDriverFactory for DBUniNeo4jFactory {
    fn database(&self) -> DBUniDatabase {
        DBUniDatabase::Neo4j
    }
    fn connect(&self, options: &DBUniConnectOptions) -> DBUniResultT<DBUniBoxedConnection> {
        if options.host.trim().is_empty() {
            return Err(missing_option("host"));
        }
        if options.port == 0 {
            return Err(missing_option("port"));
        }
        let base_url = DBUniNeo4jConnection::build_base_url(options, options.port);
        let runtime =
            new_driver_runtime().map_err(|e| DBUniError::ConnectionFailed(format!("{e}")))?;
        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| DBUniError::ConnectionFailed(format!("{e}")))?;
        Ok(Box::new(DBUniNeo4jConnection {
            client,
            runtime,
            base_url,
            user: options.user.clone(),
            password: options.password.clone(),
            database: DBUniNeo4jConnection::resolve_database(options),
        }))
    }
}
