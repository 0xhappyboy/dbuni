//! ClickHouse driver for dbuni, backed by the HTTP interface via `reqwest`.
use crate::db::{
    DBUniBoxedConnection, DBUniConnectOptions, DBUniConnection, DBUniDriverFactory, DBUniParam,
    DBUniRawRow, missing_option, new_driver_runtime,
};
use crate::types::{
    DBUniColumn, DBUniDataType, DBUniDatabase, DBUniError, DBUniPayload, DBUniResult,
    DBUniResultSet, DBUniResultT, DBUniValue,
};
/// A ClickHouse connection backed by a reqwest client and a private runtime.
pub(crate) struct DBUniClickHouseConnection {
    client: reqwest::Client,
    runtime: tokio::runtime::Runtime,
    base_url: String,
    user: String,
    password: String,
    database: String,
}
impl DBUniClickHouseConnection {
    /// Builds the ClickHouse HTTP endpoint URL.
    fn build_base_url(options: &DBUniConnectOptions, port: u16) -> String {
        let scheme = options.get_option("scheme").unwrap_or("http");
        format!("{}://{}:{}", scheme, options.host, port)
    }
    /// Escapes a string literal for ClickHouse SQL.
    fn escape_sql_string(value: &str) -> String {
        value.replace('\\', "\\\\").replace('\'', "\\'")
    }
    /// Renders a `DBUniParam` as a ClickHouse SQL literal.
    fn param_to_literal(param: &DBUniParam) -> String {
        match param {
            DBUniParam::Null => "NULL".to_string(),
            DBUniParam::Bool(v) => if *v { "1" } else { "0" }.to_string(),
            DBUniParam::Int(v) => v.to_string(),
            DBUniParam::UInt(v) => v.to_string(),
            DBUniParam::Float(v) => v.to_string(),
            DBUniParam::Text(v) => format!("'{}'", Self::escape_sql_string(v)),
            DBUniParam::Bytes(v) => {
                let hex: String = v.iter().map(|b| format!("{b:02x}")).collect();
                format!("unhex('{hex}')")
            }
            DBUniParam::Timestamp(v) => format!("toDateTime64({}, 3)", *v as f64 / 1000.0),
            DBUniParam::Json(v) => format!("'{}'", Self::escape_sql_string(v)),
        }
    }
    /// Substitutes `?` placeholders with rendered literals.
    fn render_statement(statement: &str, params: &[DBUniParam]) -> DBUniResultT<String> {
        let mut out = String::with_capacity(statement.len() + params.len() * 8);
        let mut param_iter = params.iter();
        for ch in statement.chars() {
            if ch == '?' {
                match param_iter.next() {
                    Some(p) => out.push_str(&Self::param_to_literal(p)),
                    None => {
                        return Err(DBUniError::ParameterError(
                            "not enough bound parameters for placeholders".to_string(),
                        ));
                    }
                }
            } else {
                out.push(ch);
            }
        }
        if param_iter.next().is_some() {
            return Err(DBUniError::ParameterError(
                "too many bound parameters for placeholders".to_string(),
            ));
        }
        Ok(out)
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
    /// Parses a ClickHouse JSONEachRow response into raw rows.
    fn parse_json_rows(body: &str) -> DBUniResultT<Vec<DBUniRawRow>> {
        let mut rows = Vec::new();
        for line in body.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let json: serde_json::Value = serde_json::from_str(line).map_err(|e| {
                DBUniError::Serialization(format!("invalid ClickHouse JSON row: {e}"))
            })?;
            let obj = match json.as_object() {
                Some(o) => o,
                None => continue,
            };
            let columns: Vec<String> = obj.keys().cloned().collect();
            let values: Vec<DBUniValue> = obj.values().map(Self::json_to_uni).collect();
            rows.push(DBUniRawRow::new(columns, values));
        }
        Ok(rows)
    }
    /// Infers column metadata from raw rows.
    fn infer_columns(rows: &[DBUniRawRow]) -> Vec<DBUniColumn> {
        let Some(first) = rows.first() else {
            return Vec::new();
        };
        first
            .columns
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let data_type = rows
                    .iter()
                    .filter_map(|row| row.values.get(index))
                    .find(|value| !value.is_null())
                    .map(|value| value.data_type())
                    .unwrap_or(DBUniDataType::Null);
                DBUniColumn::new(name.clone(), data_type)
            })
            .collect()
    }
    /// Builds a self-describing result from raw rows.
    fn build_result(rows: Vec<DBUniRawRow>, elapsed_ms: u64) -> DBUniResult {
        let columns = Self::infer_columns(&rows);
        let values: Vec<Vec<DBUniValue>> = rows.into_iter().map(|r| r.values).collect();
        let result_set = DBUniResultSet {
            columns,
            rows: values,
        };
        DBUniResult::new(
            DBUniDatabase::ClickHouse,
            elapsed_ms,
            None,
            DBUniPayload::Rows(result_set),
        )
    }
    /// Sends a SQL statement over HTTP and returns the raw body.
    fn send_sql(&self, sql: &str) -> DBUniResultT<String> {
        let url = format!(
            "{}/?database={}&default_format=JSONEachRow",
            self.base_url, self.database
        );
        let client = self.client.clone();
        let user = self.user.clone();
        let password = self.password.clone();
        let sql = sql.to_string();
        self.runtime.block_on(async move {
            let resp = client
                .post(&url)
                .basic_auth(user, Some(password))
                .body(sql)
                .send()
                .await
                .map_err(|e| DBUniError::Driver {
                    database: DBUniDatabase::ClickHouse,
                    code: None,
                    message: e.to_string(),
                })?;
            let status = resp.status();
            let text = resp.text().await.map_err(|e| DBUniError::Driver {
                database: DBUniDatabase::ClickHouse,
                code: None,
                message: e.to_string(),
            })?;
            if !status.is_success() {
                return Err(DBUniError::Driver {
                    database: DBUniDatabase::ClickHouse,
                    code: Some(status.as_u16().to_string()),
                    message: text,
                });
            }
            Ok(text)
        })
    }
}
impl DBUniConnection for DBUniClickHouseConnection {
    fn database(&self) -> DBUniDatabase {
        DBUniDatabase::ClickHouse
    }
    fn query_raw(&mut self, statement: &str, params: &[DBUniParam]) -> DBUniResultT<DBUniResult> {
        let start = std::time::Instant::now();
        let rows = self.query_rows(statement, params)?;
        let elapsed = start.elapsed().as_millis() as u64;
        Ok(Self::build_result(rows, elapsed))
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
        let sql = Self::render_statement(trimmed, params)?;
        let body = self.send_sql(&sql)?;
        Self::parse_json_rows(&body)
    }
    fn execute(&mut self, statement: &str, params: &[DBUniParam]) -> DBUniResultT<u64> {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            return Err(DBUniError::SyntaxError {
                statement: statement.to_string(),
                message: "empty statement".to_string(),
            });
        }
        let sql = Self::render_statement(trimmed, params)?;
        // Append the format clause only for statements that return rows.
        let lower = sql.to_lowercase();
        let is_select = lower.trim_start().starts_with("select")
            || lower.trim_start().starts_with("show")
            || lower.trim_start().starts_with("describe")
            || lower.trim_start().starts_with("with");
        if is_select {
            let body = self.send_sql(&sql)?;
            let rows = Self::parse_json_rows(&body)?;
            Ok(rows.len() as u64)
        } else {
            let _ = self.send_sql(&sql)?;
            Ok(0)
        }
    }
    fn close(&mut self) -> DBUniResultT<()> {
        // The reqwest client closes automatically when dropped.
        Ok(())
    }
}
/// Factory for the ClickHouse driver.
pub(crate) struct DBUniClickHouseFactory;
impl DBUniDriverFactory for DBUniClickHouseFactory {
    fn database(&self) -> DBUniDatabase {
        DBUniDatabase::ClickHouse
    }
    fn connect(&self, options: &DBUniConnectOptions) -> DBUniResultT<DBUniBoxedConnection> {
        if options.host.trim().is_empty() {
            return Err(missing_option("host"));
        }
        if options.port == 0 {
            return Err(missing_option("port"));
        }
        if options.database.trim().is_empty() {
            return Err(missing_option("database"));
        }
        let base_url = DBUniClickHouseConnection::build_base_url(options, options.port);
        let runtime =
            new_driver_runtime().map_err(|e| DBUniError::ConnectionFailed(format!("{e}")))?;
        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| DBUniError::ConnectionFailed(format!("{e}")))?;
        Ok(Box::new(DBUniClickHouseConnection {
            client,
            runtime,
            base_url,
            user: options.user.clone(),
            password: options.password.clone(),
            database: options.database.clone(),
        }))
    }
}
