//! Redis driver for dbuni, backed by the `redis` crate.
use crate::db::{
    DBUniBoxedConnection, DBUniConnectOptions, DBUniConnection, DBUniDriverFactory, DBUniParam,
    DBUniRawRow, missing_option, new_driver_runtime,
};
use crate::types::{
    DBUniColumn, DBUniDataType, DBUniDatabase, DBUniError, DBUniPayload, DBUniResult, DBUniResultT,
    DBUniValue,
};
use redis::aio::ConnectionManager;
use redis::{FromRedisValue, Value as RedisValue, cmd};
/// A Redis connection backed by an async connection manager and a private runtime.
pub(crate) struct DBUniRedisConnection {
    manager: ConnectionManager,
    runtime: tokio::runtime::Runtime,
    database: String,
}
impl DBUniRedisConnection {
    /// Converts a Redis reply value into a `DBUniValue`.
    fn redis_value_to_uni(value: &RedisValue) -> DBUniValue {
        match value {
            RedisValue::Nil => DBUniValue::Null,
            RedisValue::Int(v) => DBUniValue::Int(*v),
            RedisValue::BulkString(bytes) => {
                // Try UTF-8 first; fall back to raw bytes.
                match std::str::from_utf8(bytes) {
                    Ok(text) => DBUniValue::Text(text.to_string()),
                    Err(_) => DBUniValue::Bytes(bytes.clone()),
                }
            }
            RedisValue::SimpleString(text) => DBUniValue::Text(text.clone()),
            RedisValue::Okay => DBUniValue::Text("OK".to_string()),
            RedisValue::Array(items) => {
                DBUniValue::Array(items.iter().map(Self::redis_value_to_uni).collect())
            }
            RedisValue::Map(entries) => DBUniValue::Map(
                entries
                    .iter()
                    .map(|(k, v)| (Self::redis_value_to_uni(k), Self::redis_value_to_uni(v)))
                    .collect(),
            ),
            RedisValue::Set(items) => {
                DBUniValue::Array(items.iter().map(Self::redis_value_to_uni).collect())
            }
            RedisValue::Double(v) => DBUniValue::Float(*v),
            RedisValue::Boolean(v) => DBUniValue::Bool(*v),
            RedisValue::BigNumber(v) => DBUniValue::Text(v.to_string()),
            RedisValue::VerbatimString { format: _, text } => DBUniValue::Text(text.clone()),
            RedisValue::Attribute {
                data,
                attributes: _,
            } => Self::redis_value_to_uni(data),
            RedisValue::Push { kind: _, data } => {
                DBUniValue::Array(data.iter().map(Self::redis_value_to_uni).collect())
            }
            _ => DBUniValue::Null,
        }
    }
    /// Classifies a Redis reply into the matching `DBUniPayload` variant.
    fn build_payload(value: &RedisValue) -> DBUniPayload {
        match value {
            RedisValue::Nil => DBUniPayload::Empty,
            RedisValue::Okay => DBUniPayload::Ok,
            RedisValue::Int(_) | RedisValue::Double(_) | RedisValue::Boolean(_) => {
                DBUniPayload::Scalar(Self::redis_value_to_uni(value))
            }
            RedisValue::BulkString(_)
            | RedisValue::SimpleString(_)
            | RedisValue::BigNumber(_)
            | RedisValue::VerbatimString { .. } => {
                DBUniPayload::Scalar(Self::redis_value_to_uni(value))
            }
            RedisValue::Array(_) | RedisValue::Set(_) | RedisValue::Push { .. } => {
                DBUniPayload::Array(Self::flatten_array(value))
            }
            RedisValue::Map(_) => DBUniPayload::Map(Self::flatten_map(value)),
            RedisValue::Attribute { data, .. } => Self::build_payload(data),
            _ => DBUniPayload::Empty,
        }
    }
    /// Flattens a Redis array reply into a `Vec<DBUniValue>`.
    fn flatten_array(value: &RedisValue) -> Vec<DBUniValue> {
        match value {
            RedisValue::Array(items)
            | RedisValue::Set(items)
            | RedisValue::Push { data: items, .. } => {
                items.iter().map(Self::redis_value_to_uni).collect()
            }
            other => vec![Self::redis_value_to_uni(other)],
        }
    }
    /// Flattens a Redis map reply into key-value pairs.
    fn flatten_map(value: &RedisValue) -> Vec<(DBUniValue, DBUniValue)> {
        match value {
            RedisValue::Map(entries) => entries
                .iter()
                .map(|(k, v)| (Self::redis_value_to_uni(k), Self::redis_value_to_uni(v)))
                .collect(),
            _ => Vec::new(),
        }
    }
    /// Wraps a Redis reply into a single raw row for route 1.
    fn value_to_raw_row(value: &RedisValue) -> DBUniRawRow {
        DBUniRawRow::new(
            vec!["value".to_string()],
            vec![Self::redis_value_to_uni(value)],
        )
    }
    /// Infers a single column descriptor for the `"value"` column.
    #[allow(dead_code)]
    fn infer_value_column(row: &DBUniRawRow) -> Vec<DBUniColumn> {
        let data_type = row
            .values
            .first()
            .map(|v| v.data_type())
            .unwrap_or(DBUniDataType::Null);
        vec![DBUniColumn::new("value", data_type)]
    }
    /// Builds the Redis connection URL from caller-supplied options.
    fn build_url(options: &DBUniConnectOptions, port: u16) -> String {
        if options.password.is_empty() {
            format!("redis://{}:{}/{}", options.host, port, options.database)
        } else {
            format!(
                "redis://:{}@{}:{}/{}",
                options.password, options.host, port, options.database
            )
        }
    }
    /// Builds the command from a statement and bound parameters.
    fn build_command(statement: &str, params: &[DBUniParam]) -> redis::Cmd {
        let mut command = cmd(statement);
        for param in params {
            match param {
                DBUniParam::Null => {
                    command.arg("");
                }
                DBUniParam::Bool(v) => {
                    command.arg(if *v { "1" } else { "0" });
                }
                DBUniParam::Int(v) => {
                    command.arg(*v);
                }
                DBUniParam::UInt(v) => {
                    command.arg(*v);
                }
                DBUniParam::Float(v) => {
                    command.arg(*v);
                }
                DBUniParam::Text(v) => {
                    command.arg(v);
                }
                DBUniParam::Bytes(v) => {
                    command.arg(v);
                }
                DBUniParam::Timestamp(v) => {
                    command.arg(*v);
                }
                DBUniParam::Json(v) => {
                    command.arg(v);
                }
            }
        }
        command
    }
    /// Extracts the `db` index from options.
    fn resolve_database_index(options: &DBUniConnectOptions) -> DBUniResultT<u32> {
        if options.database.trim().is_empty() {
            return Err(missing_option("database"));
        }
        options
            .database
            .parse::<u32>()
            .map_err(|e| DBUniError::ParameterError(format!("invalid redis database index: {e}")))
    }
}
impl DBUniConnection for DBUniRedisConnection {
    fn database(&self) -> DBUniDatabase {
        DBUniDatabase::Redis
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
        let params_owned: Vec<DBUniParam> = params.to_vec();
        let statement_owned = trimmed.to_string();
        let mut manager = self.manager.clone();
        let value: RedisValue = self.runtime.block_on(async move {
            let command = Self::build_command(&statement_owned, &params_owned);
            command
                .query_async(&mut manager)
                .await
                .map_err(|e| DBUniError::Driver {
                    database: DBUniDatabase::Redis,
                    code: None,
                    message: e.to_string(),
                })
        })?;
        let elapsed = start.elapsed().as_millis() as u64;
        let payload = Self::build_payload(&value);
        Ok(DBUniResult::new(
            DBUniDatabase::Redis,
            elapsed,
            None,
            payload,
        ))
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
        let params_owned: Vec<DBUniParam> = params.to_vec();
        let statement_owned = trimmed.to_string();
        let mut manager = self.manager.clone();
        let value: RedisValue = self.runtime.block_on(async move {
            let command = Self::build_command(&statement_owned, &params_owned);
            command
                .query_async(&mut manager)
                .await
                .map_err(|e| DBUniError::Driver {
                    database: DBUniDatabase::Redis,
                    code: None,
                    message: e.to_string(),
                })
        })?;
        Ok(vec![Self::value_to_raw_row(&value)])
    }
    fn execute(&mut self, statement: &str, params: &[DBUniParam]) -> DBUniResultT<u64> {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            return Err(DBUniError::SyntaxError {
                statement: statement.to_string(),
                message: "empty statement".to_string(),
            });
        }
        let params_owned: Vec<DBUniParam> = params.to_vec();
        let statement_owned = trimmed.to_string();
        let mut manager = self.manager.clone();
        let affected: i64 = self.runtime.block_on(async move {
            let command = Self::build_command(&statement_owned, &params_owned);
            let value: RedisValue =
                command
                    .query_async(&mut manager)
                    .await
                    .map_err(|e| DBUniError::Driver {
                        database: DBUniDatabase::Redis,
                        code: None,
                        message: e.to_string(),
                    })?;
            // Redis commands report affected counts as integers.
            Ok::<i64, DBUniError>(i64::from_redis_value(value).unwrap_or(0))
        })?;
        Ok(affected.max(0) as u64)
    }
    fn close(&mut self) -> DBUniResultT<()> {
        // ConnectionManager closes automatically when dropped.
        Ok(())
    }
}
/// Factory for the Redis driver.
pub(crate) struct DBUniRedisFactory;
impl DBUniDriverFactory for DBUniRedisFactory {
    fn database(&self) -> DBUniDatabase {
        DBUniDatabase::Redis
    }
    fn connect(&self, options: &DBUniConnectOptions) -> DBUniResultT<DBUniBoxedConnection> {
        if options.host.trim().is_empty() {
            return Err(missing_option("host"));
        }
        if options.port == 0 {
            return Err(missing_option("port"));
        }
        let _ = DBUniRedisConnection::resolve_database_index(options)?;
        let url = DBUniRedisConnection::build_url(options, options.port);
        // Build a private runtime for this connection.
        let runtime =
            new_driver_runtime().map_err(|e| DBUniError::ConnectionFailed(format!("{e}")))?;
        let manager = runtime.block_on(async {
            let client = redis::Client::open(url.as_str())
                .map_err(|e| DBUniError::ConnectionFailed(format!("{e}")))?;
            client
                .get_connection_manager()
                .await
                .map_err(|e| DBUniError::ConnectionFailed(format!("{e}")))
        })?;
        Ok(Box::new(DBUniRedisConnection {
            manager,
            runtime,
            database: options.database.clone(),
        }))
    }
}
