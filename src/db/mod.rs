//! Database driver abstraction for dbuni.
pub(crate) mod clickhouse_client;
pub(crate) mod elasticsearch_client;
pub(crate) mod mongodb_client;
pub(crate) mod mysql_client;
pub(crate) mod neo4j_client;
pub(crate) mod postgres_client;
pub(crate) mod redis_client;
pub(crate) mod sqlite_client;
use crate::types::{DBUniDatabase, DBUniError, DBUniResult, DBUniResultT, DBUniValue};
use std::collections::HashMap;
use std::sync::Arc;
/// A single bound parameter value.
#[derive(Debug, Clone, PartialEq)]
pub enum DBUniParam {
    Null,
    Bool(bool),
    Int(i64),
    UInt(u64),
    Float(f64),
    Text(String),
    Bytes(Vec<u8>),
    Timestamp(i64),
    Json(String),
}
impl From<&str> for DBUniParam {
    fn from(value: &str) -> Self {
        DBUniParam::Text(value.to_string())
    }
}
impl From<String> for DBUniParam {
    fn from(value: String) -> Self {
        DBUniParam::Text(value)
    }
}
impl From<i64> for DBUniParam {
    fn from(value: i64) -> Self {
        DBUniParam::Int(value)
    }
}
impl From<i32> for DBUniParam {
    fn from(value: i32) -> Self {
        DBUniParam::Int(value as i64)
    }
}
impl From<u64> for DBUniParam {
    fn from(value: u64) -> Self {
        DBUniParam::UInt(value)
    }
}
impl From<f64> for DBUniParam {
    fn from(value: f64) -> Self {
        DBUniParam::Float(value)
    }
}
impl From<bool> for DBUniParam {
    fn from(value: bool) -> Self {
        DBUniParam::Bool(value)
    }
}
impl From<Vec<u8>> for DBUniParam {
    fn from(value: Vec<u8>) -> Self {
        DBUniParam::Bytes(value)
    }
}
/// Extensible connection parameters.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DBUniConnectOptions {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    /// File path for embedded databases such as SQLite.
    pub path: String,
    /// Optional driver selector, e.g. "native", "mariadb".
    pub driver: Option<String>,
    /// Extra driver-specific key-value options.
    pub extra: Vec<(String, String)>,
}
impl DBUniConnectOptions {
    pub fn new(
        host: impl Into<String>,
        port: u16,
        user: impl Into<String>,
        password: impl Into<String>,
        database: impl Into<String>,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            user: user.into(),
            password: password.into(),
            database: database.into(),
            path: String::new(),
            driver: None,
            extra: Vec::new(),
        }
    }
    /// Creates options for an embedded database, identified by a file path instead of a host and port.
    pub fn new_embedded(path: impl Into<String>) -> Self {
        Self {
            host: String::new(),
            port: 0,
            user: String::new(),
            password: String::new(),
            database: String::new(),
            path: path.into(),
            driver: None,
            extra: Vec::new(),
        }
    }
    /// Sets the file path for an embedded database.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }
    pub fn with_driver(mut self, driver: impl Into<String>) -> Self {
        self.driver = Some(driver.into());
        self
    }
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.push((key.into(), value.into()));
        self
    }
    pub fn get_option(&self, key: &str) -> Option<&str> {
        self.extra
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
    /// Builds a sqlx-compatible connection URL for network databases.
    pub fn to_url(&self) -> String {
        let scheme = match self.driver.as_deref() {
            Some("mariadb") => "mysql",
            Some(other) => other,
            None => "mysql",
        };
        format!(
            "{}://{}:{}@{}:{}/{}",
            scheme, self.user, self.password, self.host, self.port, self.database
        )
    }
}
/// A raw row produced by a driver.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DBUniRawRow {
    pub columns: Vec<String>,
    pub values: Vec<DBUniValue>,
}
impl DBUniRawRow {
    pub fn new(columns: Vec<String>, values: Vec<DBUniValue>) -> Self {
        Self { columns, values }
    }
    pub fn get(&self, index: usize) -> Option<&DBUniValue> {
        self.values.get(index)
    }
    pub fn get_by_name(&self, name: &str) -> Option<&DBUniValue> {
        self.columns
            .iter()
            .position(|c| c == name)
            .and_then(|i| self.values.get(i))
    }
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c == name)
    }
}
/// A type that can build itself from a raw row.
pub trait DBUniFromRow: Sized {
    fn from_row(row: &DBUniRawRow) -> DBUniResultT<Self>;
}
/// The unified connection trait.
pub(crate) trait DBUniConnection: Send + Sync {
    fn database(&self) -> DBUniDatabase;
    /// Route 2: self-describing result.
    fn query_raw(&mut self, statement: &str, params: &[DBUniParam]) -> DBUniResultT<DBUniResult>;
    /// Shared primitive behind both routes.
    fn query_rows(
        &mut self,
        statement: &str,
        params: &[DBUniParam],
    ) -> DBUniResultT<Vec<DBUniRawRow>>;
    /// Executes a statement without rows.
    fn execute(&mut self, statement: &str, params: &[DBUniParam]) -> DBUniResultT<u64>;
    fn close(&mut self) -> DBUniResultT<()>;
    fn ping(&mut self) -> DBUniResultT<()> {
        let _ = self.query_rows("SELECT 1", &[])?;
        Ok(())
    }
}
/// A boxed, thread-safe connection. Internal to the crate.
pub(crate) type DBUniBoxedConnection = Box<dyn DBUniConnection>;
/// A driver factory. Internal to the crate.
pub(crate) trait DBUniDriverFactory: Send + Sync {
    fn database(&self) -> DBUniDatabase;
    fn connect(&self, options: &DBUniConnectOptions) -> DBUniResultT<DBUniBoxedConnection>;
}
/// The driver registry. Internal to the crate.
#[derive(Default)]
pub(crate) struct DBUniRegistry {
    factories: HashMap<DBUniDatabase, Arc<dyn DBUniDriverFactory>>,
}
impl DBUniRegistry {
    pub(crate) fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }
    pub(crate) fn register(&mut self, factory: Arc<dyn DBUniDriverFactory>) {
        self.factories.insert(factory.database(), factory);
    }
    pub(crate) fn is_registered(&self, database: DBUniDatabase) -> bool {
        self.factories.contains_key(&database)
    }
    pub(crate) fn connect(
        &self,
        database: DBUniDatabase,
        options: &DBUniConnectOptions,
    ) -> DBUniResultT<DBUniBoxedConnection> {
        let factory = self.factories.get(&database).ok_or_else(|| {
            DBUniError::Unsupported(format!("no driver registered for {database}"))
        })?;
        factory.connect(options)
    }
    /// Registers every driver that is compiled in.
    pub(crate) fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Arc::new(mysql_client::DBUniMySqlFactory));
        registry.register(Arc::new(redis_client::DBUniRedisFactory));
        registry.register(Arc::new(postgres_client::DBUniPostgresFactory));
        registry.register(Arc::new(sqlite_client::DBUniSqliteFactory));
        registry.register(Arc::new(mongodb_client::DBUniMongoDbFactory));
        registry.register(Arc::new(clickhouse_client::DBUniClickHouseFactory));
        registry.register(Arc::new(elasticsearch_client::DBUniElasticsearchFactory));
        registry.register(Arc::new(neo4j_client::DBUniNeo4jFactory));
        registry
    }
}
/// The lifecycle state of a [`DBUniClient`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DBUniConnectionState {
    /// The client is connected and ready for operations.
    Connected,
    /// The client is in the middle of closing.
    Closing,
    /// The client has been closed and can no longer be used.
    Closed,
    /// The client failed and is no longer usable.
    Failed,
}
impl DBUniConnectionState {
    pub const fn as_key(&self) -> &'static str {
        match self {
            DBUniConnectionState::Connected => "connected",
            DBUniConnectionState::Closing => "closing",
            DBUniConnectionState::Closed => "closed",
            DBUniConnectionState::Failed => "failed",
        }
    }
    pub const fn as_str(&self) -> &'static str {
        match self {
            DBUniConnectionState::Connected => "Connected",
            DBUniConnectionState::Closing => "Closing",
            DBUniConnectionState::Closed => "Closed",
            DBUniConnectionState::Failed => "Failed",
        }
    }
    pub const fn is_usable(&self) -> bool {
        matches!(self, DBUniConnectionState::Connected)
    }
}
impl std::fmt::Display for DBUniConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
/// The single public database client.
pub struct DBUniClient {
    database: DBUniDatabase,
    options: DBUniConnectOptions,
    state: DBUniConnectionState,
    inner: DBUniBoxedConnection,
}
impl DBUniClient {
    /// Connects to the given database using the given options.
    pub fn connect(database: DBUniDatabase, options: &DBUniConnectOptions) -> DBUniResultT<Self> {
        let registry = DBUniRegistry::with_defaults();
        let inner = registry.connect(database, options)?;
        Ok(Self {
            database,
            options: options.clone(),
            state: DBUniConnectionState::Connected,
            inner,
        })
    }
    pub fn database(&self) -> DBUniDatabase {
        self.database
    }
    pub fn options(&self) -> &DBUniConnectOptions {
        &self.options
    }
    pub fn state(&self) -> DBUniConnectionState {
        self.state
    }
    pub fn is_connected(&self) -> bool {
        self.state.is_usable()
    }
    fn ensure_usable(&self) -> DBUniResultT<()> {
        if self.state.is_usable() {
            Ok(())
        } else {
            Err(DBUniError::ConnectionLost(format!(
                "client is {} and cannot run operations",
                self.state.as_str()
            )))
        }
    }
    pub fn query_raw(
        &mut self,
        statement: &str,
        params: &[DBUniParam],
    ) -> DBUniResultT<DBUniResult> {
        self.ensure_usable()?;
        self.inner.query_raw(statement, params)
    }
    pub fn query_rows(
        &mut self,
        statement: &str,
        params: &[DBUniParam],
    ) -> DBUniResultT<Vec<DBUniRawRow>> {
        self.ensure_usable()?;
        self.inner.query_rows(statement, params)
    }
    pub fn query_one_as<T>(
        &mut self,
        statement: &str,
        params: &[DBUniParam],
    ) -> DBUniResultT<Option<T>>
    where
        T: DBUniFromRow,
    {
        self.ensure_usable()?;
        let rows = self.inner.query_rows(statement, params)?;
        match rows.first() {
            Some(row) => Ok(Some(T::from_row(row)?)),
            None => Ok(None),
        }
    }
    pub fn query_all_as<T>(
        &mut self,
        statement: &str,
        params: &[DBUniParam],
    ) -> DBUniResultT<Vec<T>>
    where
        T: DBUniFromRow,
    {
        self.ensure_usable()?;
        let rows = self.inner.query_rows(statement, params)?;
        let mut out = Vec::with_capacity(rows.len());
        for row in &rows {
            out.push(T::from_row(row)?);
        }
        Ok(out)
    }
    pub fn execute(&mut self, statement: &str, params: &[DBUniParam]) -> DBUniResultT<u64> {
        self.ensure_usable()?;
        self.inner.execute(statement, params)
    }
    pub fn ping(&mut self) -> DBUniResultT<()> {
        self.ensure_usable()?;
        self.inner.ping()
    }
    pub fn close(&mut self) -> DBUniResultT<()> {
        if matches!(self.state, DBUniConnectionState::Closed) {
            return Ok(());
        }
        self.state = DBUniConnectionState::Closing;
        match self.inner.close() {
            Ok(()) => {
                self.state = DBUniConnectionState::Closed;
                Ok(())
            }
            Err(e) => {
                self.state = DBUniConnectionState::Failed;
                Err(e)
            }
        }
    }
}
impl Drop for DBUniClient {
    fn drop(&mut self) {
        if self.state.is_usable() {
            let _ = self.inner.close();
            self.state = DBUniConnectionState::Closed;
        }
    }
}
/// Builds a private runtime for a driver.
pub(crate) fn new_driver_runtime() -> std::io::Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .thread_name("dbuni-driver")
        .build()
}
/// Returns an error for a missing required connection option.
pub(crate) fn missing_option(name: &str) -> DBUniError {
    DBUniError::ParameterError(format!("missing required connection option: {name}"))
}
