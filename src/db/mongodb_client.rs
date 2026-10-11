//! MongoDB driver for dbuni, backed by the `mongodb` crate.
use crate::db::{
    DBUniBoxedConnection, DBUniConnectOptions, DBUniConnection, DBUniDriverFactory, DBUniParam,
    DBUniRawRow, missing_option, new_driver_runtime,
};
use crate::types::{
    DBUniDatabase, DBUniError, DBUniPayload, DBUniResult, DBUniResultT, DBUniValue,
};
use futures_util::stream::TryStreamExt;
use mongodb::bson::{Bson, Document};
use mongodb::options::ClientOptions;
use mongodb::{Client, Collection};
/// Converts a `mongodb::error::Error` into a `DBUniError`.
fn mongo_err(e: mongodb::error::Error) -> DBUniError {
    DBUniError::Driver {
        database: DBUniDatabase::MongoDb,
        code: None,
        message: e.to_string(),
    }
}
/// A MongoDB connection backed by a client and a private runtime.
pub(crate) struct DBUniMongoDbConnection {
    client: Client,
    runtime: tokio::runtime::Runtime,
    database: String,
}
impl DBUniMongoDbConnection {
    /// Converts a BSON value into a `DBUniValue`.
    fn bson_to_uni(value: &Bson) -> DBUniValue {
        match value {
            Bson::Null | Bson::Undefined => DBUniValue::Null,
            Bson::Boolean(v) => DBUniValue::Bool(*v),
            Bson::Int32(v) => DBUniValue::Int(*v as i64),
            Bson::Int64(v) => DBUniValue::Int(*v),
            Bson::Double(v) => DBUniValue::Float(*v),
            Bson::String(v) => DBUniValue::Text(v.clone()),
            Bson::Binary(b) => DBUniValue::Bytes(b.bytes.clone()),
            Bson::DateTime(dt) => DBUniValue::Timestamp(dt.timestamp_millis()),
            Bson::Array(items) => DBUniValue::Array(items.iter().map(Self::bson_to_uni).collect()),
            Bson::Document(doc) => DBUniValue::Map(
                doc.iter()
                    .map(|(k, v)| (DBUniValue::Text(k.clone()), Self::bson_to_uni(v)))
                    .collect(),
            ),
            other => DBUniValue::Text(format!("{other:?}")),
        }
    }
    /// Converts a document into a `DBUniValue::Map`.
    fn document_to_uni(doc: &Document) -> DBUniValue {
        DBUniValue::Map(
            doc.iter()
                .map(|(k, v)| (DBUniValue::Text(k.clone()), Self::bson_to_uni(v)))
                .collect(),
        )
    }
    /// Converts a document into a flat raw row.
    fn document_to_raw_row(doc: &Document) -> DBUniRawRow {
        let columns: Vec<String> = doc.keys().cloned().collect();
        let values: Vec<DBUniValue> = doc.values().map(Self::bson_to_uni).collect();
        DBUniRawRow::new(columns, values)
    }
    /// Parses a `DBUniParam` into a BSON value.
    fn param_to_bson(param: &DBUniParam) -> Bson {
        match param {
            DBUniParam::Null => Bson::Null,
            DBUniParam::Bool(v) => Bson::Boolean(*v),
            DBUniParam::Int(v) => Bson::Int64(*v),
            DBUniParam::UInt(v) => Bson::Int64(*v as i64),
            DBUniParam::Float(v) => Bson::Double(*v),
            DBUniParam::Text(v) => Bson::String(v.clone()),
            DBUniParam::Bytes(v) => Bson::Binary(mongodb::bson::Binary {
                subtype: mongodb::bson::spec::BinarySubtype::Generic,
                bytes: v.clone(),
            }),
            DBUniParam::Timestamp(v) => Bson::DateTime(mongodb::bson::DateTime::from_millis(*v)),
            DBUniParam::Json(v) => {
                match serde_json::from_str::<serde_json::Value>(v) {
                    Ok(json) => mongodb::bson::to_bson(&json).unwrap_or(Bson::Null),
                    Err(_) => Bson::String(v.clone()),
                }
            }
        }
    }
    /// Parses the statement into a BSON document.
    fn parse_statement(statement: &str) -> DBUniResultT<Document> {
        let json: serde_json::Value =
            serde_json::from_str(statement.trim()).map_err(|e| DBUniError::SyntaxError {
                statement: statement.to_string(),
                message: format!("invalid MongoDB statement JSON: {e}"),
            })?;
        mongodb::bson::to_document(&json).map_err(|e| DBUniError::SyntaxError {
            statement: statement.to_string(),
            message: format!("invalid MongoDB statement BSON: {e}"),
        })
    }
    /// Extracts a string field from a document, or returns an error.
    fn required_str(doc: &Document, key: &str) -> DBUniResultT<String> {
        doc.get_str(key)
            .map(|s| s.to_string())
            .map_err(|_| DBUniError::ParameterError(format!("missing required field: {key}")))
    }
    /// Extracts a nested document field, defaulting to an empty one.
    fn optional_doc(doc: &Document, key: &str) -> Document {
        doc.get_document(key).cloned().unwrap_or_default()
    }
    /// Extracts a nested document from the first bound parameter.
    fn doc_from_params(params: &[DBUniParam], key: &str) -> DBUniResultT<Document> {
        let Some(param) = params.first() else {
            return Err(missing_option(key));
        };
        let text = match param {
            DBUniParam::Json(text) | DBUniParam::Text(text) => text,
            _ => {
                return Err(DBUniError::ParameterError(format!(
                    "{key} must be a JSON object"
                )));
            }
        };
        let json: serde_json::Value = serde_json::from_str(text)
            .map_err(|e| DBUniError::ParameterError(format!("invalid JSON for {key}: {e}")))?;
        mongodb::bson::to_document(&json)
            .map_err(|e| DBUniError::ParameterError(format!("invalid BSON for {key}: {e}")))
    }
    /// Resolves the database name from options.
    fn resolve_database(options: &DBUniConnectOptions) -> DBUniResultT<String> {
        if options.database.trim().is_empty() {
            return Err(missing_option("database"));
        }
        Ok(options.database.clone())
    }
    /// Builds the MongoDB connection URI from caller-supplied options.
    fn build_uri(options: &DBUniConnectOptions, port: u16) -> String {
        if options.user.trim().is_empty() {
            format!("mongodb://{}:{}", options.host, port)
        } else {
            format!(
                "mongodb://{}:{}@{}:{}",
                options.user, options.password, options.host, port
            )
        }
    }
}
impl DBUniConnection for DBUniMongoDbConnection {
    fn database(&self) -> DBUniDatabase {
        DBUniDatabase::MongoDb
    }
    fn query_raw(&mut self, statement: &str, params: &[DBUniParam]) -> DBUniResultT<DBUniResult> {
        let start = std::time::Instant::now();
        let spec = Self::parse_statement(statement)?;
        let collection_name = Self::required_str(&spec, "collection")?;
        let filter = Self::optional_doc(&spec, "filter");
        let collection: Collection<Document> = self
            .client
            .database(&self.database)
            .collection(&collection_name);
        let _ = params;
        // `Find` is not a `Stream`; it must be awaited first to
        // produce a `Cursor`, and the cursor is then collected.
        let docs: Vec<Document> = self.runtime.block_on(async move {
            let mut find = collection.find(filter);
            if let Some(limit) = spec.get_i64("limit").ok() {
                find = find.limit(limit);
            }
            let cursor = find.await.map_err(mongo_err)?;
            cursor.try_collect().await.map_err(mongo_err)
        })?;
        let mut documents: Vec<DBUniValue> = Vec::new();
        for doc in &docs {
            documents.push(Self::document_to_uni(doc));
        }
        let elapsed = start.elapsed().as_millis() as u64;
        let affected = Some(documents.len() as u64);
        Ok(DBUniResult::new(
            DBUniDatabase::MongoDb,
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
        let collection_name = Self::required_str(&spec, "collection")?;
        let filter = Self::optional_doc(&spec, "filter");
        let collection: Collection<Document> = self
            .client
            .database(&self.database)
            .collection(&collection_name);
        let _ = params;
        let docs: Vec<Document> = self.runtime.block_on(async move {
            let mut find = collection.find(filter);
            if let Some(limit) = spec.get_i64("limit").ok() {
                find = find.limit(limit);
            }
            let cursor = find.await.map_err(mongo_err)?;
            cursor.try_collect().await.map_err(mongo_err)
        })?;
        let mut rows: Vec<DBUniRawRow> = Vec::new();
        for doc in &docs {
            rows.push(Self::document_to_raw_row(doc));
        }
        Ok(rows)
    }
    fn execute(&mut self, statement: &str, params: &[DBUniParam]) -> DBUniResultT<u64> {
        let spec = Self::parse_statement(statement)?;
        let op = spec.get_str("op").unwrap_or("insert").to_string();
        let collection_name = Self::required_str(&spec, "collection")?;
        let collection: Collection<Document> = self
            .client
            .database(&self.database)
            .collection(&collection_name);
        let filter = Self::optional_doc(&spec, "filter");
        let affected = self.runtime.block_on(async move {
            match op.as_str() {
                "insert" => {
                    let insert_doc = Self::doc_from_params(params, "document")?;
                    let result = collection.insert_one(insert_doc).await.map_err(mongo_err)?;
                    Ok::<u64, DBUniError>(if result.inserted_id != Bson::Null {
                        1
                    } else {
                        0
                    })
                }
                "update" => {
                    let update_doc = Self::doc_from_params(params, "update")?;
                    let result = collection
                        .update_one(filter, update_doc)
                        .await
                        .map_err(mongo_err)?;
                    Ok(result.modified_count)
                }
                "delete" => {
                    let result = collection.delete_many(filter).await.map_err(mongo_err)?;
                    Ok(result.deleted_count)
                }
                other => Err(DBUniError::Unsupported(format!(
                    "unsupported MongoDB op: {other}"
                ))),
            }
        })?;
        Ok(affected)
    }
    fn close(&mut self) -> DBUniResultT<()> {
        // The MongoDB client closes automatically when dropped.
        Ok(())
    }
}
/// Factory for the MongoDB driver.
pub(crate) struct DBUniMongoDbFactory;
impl DBUniDriverFactory for DBUniMongoDbFactory {
    fn database(&self) -> DBUniDatabase {
        DBUniDatabase::MongoDb
    }
    fn connect(&self, options: &DBUniConnectOptions) -> DBUniResultT<DBUniBoxedConnection> {
        if options.host.trim().is_empty() {
            return Err(missing_option("host"));
        }
        if options.port == 0 {
            return Err(missing_option("port"));
        }
        let database = DBUniMongoDbConnection::resolve_database(options)?;
        let uri = DBUniMongoDbConnection::build_uri(options, options.port);
        let runtime =
            new_driver_runtime().map_err(|e| DBUniError::ConnectionFailed(format!("{e}")))?;
        let client = runtime
            .block_on(async {
                let client_options = ClientOptions::parse(uri.as_str()).await?;
                Client::with_options(client_options)
            })
            .map_err(|e| DBUniError::ConnectionFailed(format!("{e}")))?;
        Ok(Box::new(DBUniMongoDbConnection {
            client,
            runtime,
            database,
        }))
    }
}
