//! PostgreSQL driver for dbuni, backed by `sqlx`.
use crate::db::{
    missing_option, new_driver_runtime, DBUniBoxedConnection, DBUniConnectOptions,
    DBUniConnection, DBUniDriverFactory, DBUniParam, DBUniRawRow,
};
use crate::types::{
    DBUniColumn, DBUniDataType, DBUniDatabase, DBUniError, DBUniPayload, DBUniResult,
    DBUniResultSet, DBUniResultT, DBUniValue,
};
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::{Column, Row};
/// A PostgreSQL connection backed by a sqlx pool and a private runtime.
pub(crate) struct DBUniPostgresConnection {
    pool: PgPool,
    runtime: tokio::runtime::Runtime,
    database: String,
}
impl DBUniPostgresConnection {
    /// Converts a sqlx row into a `DBUniRawRow`.
    fn row_to_raw(row: &PgRow) -> DBUniRawRow {
        let columns: Vec<String> = row.columns().iter().map(|c| c.name().to_string()).collect();
        let values: Vec<DBUniValue> = (0..columns.len())
            .map(|i| Self::value_at(row, i))
            .collect();
        DBUniRawRow::new(columns, values)
    }
    /// Extracts a column value by index, trying common types in order.
    fn value_at(row: &PgRow, index: usize) -> DBUniValue {
        // Try i64
        if let Ok(v) = row.try_get::<Option<i64>, _>(index) {
            return v.map(DBUniValue::Int).unwrap_or(DBUniValue::Null);
        }
        // Try i32
        if let Ok(v) = row.try_get::<Option<i32>, _>(index) {
            return v
                .map(|n| DBUniValue::Int(n as i64))
                .unwrap_or(DBUniValue::Null);
        }
        // Try f64
        if let Ok(v) = row.try_get::<Option<f64>, _>(index) {
            return v.map(DBUniValue::Float).unwrap_or(DBUniValue::Null);
        }
        // Try bool
        if let Ok(v) = row.try_get::<Option<bool>, _>(index) {
            return v.map(DBUniValue::Bool).unwrap_or(DBUniValue::Null);
        }
        // Try String
        if let Ok(v) = row.try_get::<Option<String>, _>(index) {
            return v.map(DBUniValue::Text).unwrap_or(DBUniValue::Null);
        }
        // Try Vec<u8>
        if let Ok(v) = row.try_get::<Option<Vec<u8>>, _>(index) {
            return v.map(DBUniValue::Bytes).unwrap_or(DBUniValue::Null);
        }
        DBUniValue::Null
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
            DBUniDatabase::PostgreSQL,
            elapsed_ms,
            None,
            DBUniPayload::Rows(result_set),
        )
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
    /// Binds a `DBUniParam` onto a sqlx query.
    fn bind_param<'q>(
        query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
        param: &'q DBUniParam,
    ) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
        match param {
            DBUniParam::Null => query.bind(Option::<String>::None),
            DBUniParam::Bool(v) => query.bind(*v),
            DBUniParam::Int(v) => query.bind(*v),
            DBUniParam::UInt(v) => query.bind(*v as i64),
            DBUniParam::Float(v) => query.bind(*v),
            DBUniParam::Text(v) => query.bind(v.clone()),
            DBUniParam::Bytes(v) => query.bind(v.clone()),
            DBUniParam::Timestamp(v) => query.bind(*v),
            DBUniParam::Json(v) => query.bind(v.clone()),
        }
    }
    /// Builds the PostgreSQL connection URL from caller-supplied options.
    fn build_url(options: &DBUniConnectOptions, port: u16) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            options.user, options.password, options.host, port, options.database
        )
    }
    /// Resolves the pool size from caller-supplied options.
    fn resolve_max_connections(options: &DBUniConnectOptions) -> DBUniResultT<u32> {
        let raw = options
            .get_option("max_connections")
            .ok_or_else(|| missing_option("max_connections"))?;
        raw.parse::<u32>()
            .map_err(|e| DBUniError::ParameterError(format!("invalid max_connections: {e}")))
    }
}
impl DBUniConnection for DBUniPostgresConnection {
    fn database(&self) -> DBUniDatabase {
        DBUniDatabase::PostgreSQL
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
        let pool = self.pool.clone();
        let sql = trimmed.to_string();
        let params: Vec<DBUniParam> = params.to_vec();
        let rows = self
            .runtime
            .block_on(async move {
                let mut query = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()));
                for p in &params {
                    query = Self::bind_param(query, p);
                }
                query.fetch_all(&pool).await
            })
            .map_err(|e| DBUniError::Driver {
                database: DBUniDatabase::PostgreSQL,
                code: None,
                message: e.to_string(),
            })?;
        Ok(rows.iter().map(Self::row_to_raw).collect())
    }
    fn execute(&mut self, statement: &str, params: &[DBUniParam]) -> DBUniResultT<u64> {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            return Err(DBUniError::SyntaxError {
                statement: statement.to_string(),
                message: "empty statement".to_string(),
            });
        }
        let pool = self.pool.clone();
        let sql = trimmed.to_string();
        let params: Vec<DBUniParam> = params.to_vec();
        let result = self
            .runtime
            .block_on(async move {
                let mut query = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()));
                for p in &params {
                    query = Self::bind_param(query, p);
                }
                query.execute(&pool).await
            })
            .map_err(|e| DBUniError::Driver {
                database: DBUniDatabase::PostgreSQL,
                code: None,
                message: e.to_string(),
            })?;
        Ok(result.rows_affected())
    }
    fn close(&mut self) -> DBUniResultT<()> {
        let pool = self.pool.clone();
        self.runtime.block_on(async move { pool.close().await });
        Ok(())
    }
}
/// Factory for the PostgreSQL driver.
pub(crate) struct DBUniPostgresFactory;
impl DBUniDriverFactory for DBUniPostgresFactory {
    fn database(&self) -> DBUniDatabase {
        DBUniDatabase::PostgreSQL
    }
    fn connect(&self, options: &DBUniConnectOptions) -> DBUniResultT<DBUniBoxedConnection> {
        if options.host.trim().is_empty() {
            return Err(missing_option("host"));
        }
        if options.user.trim().is_empty() {
            return Err(missing_option("user"));
        }
        if options.port == 0 {
            return Err(missing_option("port"));
        }
        let max_connections = DBUniPostgresConnection::resolve_max_connections(options)?;
        let url = DBUniPostgresConnection::build_url(options, options.port);
        let runtime =
            new_driver_runtime().map_err(|e| DBUniError::ConnectionFailed(format!("{e}")))?;
        let pool = runtime
            .block_on(async {
                PgPoolOptions::new()
                    .max_connections(max_connections)
                    .connect(&url)
                    .await
            })
            .map_err(|e| DBUniError::ConnectionFailed(format!("{e}")))?;
        let conn = DBUniPostgresConnection {
            pool,
            runtime,
            database: options.database.clone(),
        };
        Ok(Box::new(conn))
    }
}