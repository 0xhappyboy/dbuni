//! Core database type definitions for dbuni.
use std::fmt;
/// The broad category a database belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DatabaseCategory {
    /// Relational / SQL databases (MySQL, PostgreSQL, SQLite, ...).
    Sql,
    /// Non-relational / NoSQL databases (MongoDB, Redis, Cassandra, ...).
    NoSql,
    /// Cloud-hosted / managed databases (Snowflake, BigQuery, Redshift, ...).
    Cloud,
    /// Embedded / in-process databases (DuckDB embedded, H2, Derby, ...).
    Embedded,
    /// Time-series databases (InfluxDB, TimescaleDB, ...).
    TimeSeries,
    /// Graph databases (Neo4j, ArangoDB, ...).
    Graph,
    /// Search engines / document stores used for full-text search.
    Search,
    /// Key-value stores (Redis, Memcached, ...).
    KeyValue,
    /// Wide-column / column-family stores (Cassandra, HBase, ...).
    WideColumn,
    /// Object storage used as a database backend (S3, MinIO, ...).
    ObjectStorage,
    /// Message-queue / streaming backends that expose a database-like API.
    MessageQueue,
    /// Anything that does not fit the categories above.
    Other,
}
impl DatabaseCategory {
    /// Returns the canonical lowercase key for this category.
    pub const fn as_key(&self) -> &'static str {
        match self {
            DatabaseCategory::Sql => "sql",
            DatabaseCategory::NoSql => "nosql",
            DatabaseCategory::Cloud => "cloud",
            DatabaseCategory::Embedded => "embedded",
            DatabaseCategory::TimeSeries => "timeseries",
            DatabaseCategory::Graph => "graph",
            DatabaseCategory::Search => "search",
            DatabaseCategory::KeyValue => "keyvalue",
            DatabaseCategory::WideColumn => "widecolumn",
            DatabaseCategory::ObjectStorage => "objectstorage",
            DatabaseCategory::MessageQueue => "messagequeue",
            DatabaseCategory::Other => "other",
        }
    }
    /// Returns a human-readable English label.
    pub const fn as_str(&self) -> &'static str {
        match self {
            DatabaseCategory::Sql => "SQL",
            DatabaseCategory::NoSql => "NoSQL",
            DatabaseCategory::Cloud => "Cloud",
            DatabaseCategory::Embedded => "Embedded",
            DatabaseCategory::TimeSeries => "Time-Series",
            DatabaseCategory::Graph => "Graph",
            DatabaseCategory::Search => "Search",
            DatabaseCategory::KeyValue => "Key-Value",
            DatabaseCategory::WideColumn => "Wide-Column",
            DatabaseCategory::ObjectStorage => "Object Storage",
            DatabaseCategory::MessageQueue => "Message Queue",
            DatabaseCategory::Other => "Other",
        }
    }
}
impl fmt::Display for DatabaseCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
/// A concrete database engine supported by dbuni.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DatabaseType {
    MySql,
    MariaDb,
    PostgreSQL,
    SQLite,
    SqlServer,
    Oracle,
    Db2,
    ClickHouse,
    DuckDb,
    Firebird,
    H2,
    Derby,
    Hsqldb,
    Sybase,
    Teradata,
    Vertica,
    Greenplum,
    CockroachDb,
    TiDb,
    YugabyteDb,
    SingleStore,
    Snowflake,
    Redshift,
    BigQuery,
    Databricks,
    Trino,
    Presto,
    Impala,
    Hive,
    Phoenix,
    MongoDb,
    Redis,
    Cassandra,
    ScyllaDb,
    Neo4j,
    CouchDb,
    Couchbase,
    Elasticsearch,
    OpenSearch,
    ArangoDb,
    OrientDb,
    DynamoDb,
    CosmosDb,
    Firebase,
    Riak,
    Memcached,
    Etcd,
    Consul,
    HBase,
    Accumulo,
    InfluxDb,
    TimescaleDb,
    QuestDb,
    Prometheus,
    VictoriaMetrics,
    Supabase,
    PlanetScale,
    Neon,
    FaunaDb,
    Airtable,
    Notion,
    DuckDbEmbedded,
    SqliteEmbedded,
    RocksDb,
    Lmdb,
    Sled,
    Redb,
    S3,
    Minio,
    Kafka,
    RabbitMq,
    Nats,
    Pulsar,
    Other,
}
impl DatabaseType {
    /// Returns the broad category this engine belongs to.
    pub const fn category(&self) -> DatabaseCategory {
        match self {
            // Relational / SQL
            DatabaseType::MySql
            | DatabaseType::MariaDb
            | DatabaseType::PostgreSQL
            | DatabaseType::SQLite
            | DatabaseType::SqlServer
            | DatabaseType::Oracle
            | DatabaseType::Db2
            | DatabaseType::ClickHouse
            | DatabaseType::DuckDb
            | DatabaseType::Firebird
            | DatabaseType::H2
            | DatabaseType::Derby
            | DatabaseType::Hsqldb
            | DatabaseType::Sybase
            | DatabaseType::Teradata
            | DatabaseType::Vertica
            | DatabaseType::Greenplum
            | DatabaseType::CockroachDb
            | DatabaseType::TiDb
            | DatabaseType::YugabyteDb
            | DatabaseType::SingleStore
            | DatabaseType::Trino
            | DatabaseType::Presto
            | DatabaseType::Impala
            | DatabaseType::Hive
            | DatabaseType::Phoenix => DatabaseCategory::Sql,
            // Cloud / managed warehouses
            DatabaseType::Snowflake
            | DatabaseType::Redshift
            | DatabaseType::BigQuery
            | DatabaseType::Databricks => DatabaseCategory::Cloud,
            // Document / key-value / wide-column / graph
            DatabaseType::MongoDb
            | DatabaseType::CouchDb
            | DatabaseType::Couchbase
            | DatabaseType::DynamoDb
            | DatabaseType::CosmosDb
            | DatabaseType::Firebase
            | DatabaseType::Riak
            | DatabaseType::FaunaDb
            | DatabaseType::Airtable
            | DatabaseType::Notion => DatabaseCategory::NoSql,
            DatabaseType::Redis
            | DatabaseType::Memcached
            | DatabaseType::Etcd
            | DatabaseType::Consul => DatabaseCategory::KeyValue,
            DatabaseType::Cassandra
            | DatabaseType::ScyllaDb
            | DatabaseType::HBase
            | DatabaseType::Accumulo => DatabaseCategory::WideColumn,
            DatabaseType::Neo4j | DatabaseType::ArangoDb | DatabaseType::OrientDb => {
                DatabaseCategory::Graph
            }
            DatabaseType::Elasticsearch | DatabaseType::OpenSearch => DatabaseCategory::Search,
            DatabaseType::InfluxDb
            | DatabaseType::TimescaleDb
            | DatabaseType::QuestDb
            | DatabaseType::Prometheus
            | DatabaseType::VictoriaMetrics => DatabaseCategory::TimeSeries,
            // Cloud databases (non-warehouse)
            DatabaseType::Supabase | DatabaseType::PlanetScale | DatabaseType::Neon => {
                DatabaseCategory::Cloud
            }
            // Embedded
            DatabaseType::DuckDbEmbedded
            | DatabaseType::SqliteEmbedded
            | DatabaseType::RocksDb
            | DatabaseType::Lmdb
            | DatabaseType::Sled
            | DatabaseType::Redb => DatabaseCategory::Embedded,
            // Object storage
            DatabaseType::S3 | DatabaseType::Minio => DatabaseCategory::ObjectStorage,
            // Message queue / streaming
            DatabaseType::Kafka
            | DatabaseType::RabbitMq
            | DatabaseType::Nats
            | DatabaseType::Pulsar => DatabaseCategory::MessageQueue,
            DatabaseType::Other => DatabaseCategory::Other,
        }
    }
    /// Returns the canonical lowercase key for this engine.
    pub const fn as_key(&self) -> &'static str {
        match self {
            DatabaseType::MySql => "mysql",
            DatabaseType::MariaDb => "mariadb",
            DatabaseType::PostgreSQL => "postgresql",
            DatabaseType::SQLite => "sqlite",
            DatabaseType::SqlServer => "sqlserver",
            DatabaseType::Oracle => "oracle",
            DatabaseType::Db2 => "db2",
            DatabaseType::ClickHouse => "clickhouse",
            DatabaseType::DuckDb => "duckdb",
            DatabaseType::Firebird => "firebird",
            DatabaseType::H2 => "h2",
            DatabaseType::Derby => "derby",
            DatabaseType::Hsqldb => "hsqldb",
            DatabaseType::Sybase => "sybase",
            DatabaseType::Teradata => "teradata",
            DatabaseType::Vertica => "vertica",
            DatabaseType::Greenplum => "greenplum",
            DatabaseType::CockroachDb => "cockroachdb",
            DatabaseType::TiDb => "tidb",
            DatabaseType::YugabyteDb => "yugabytedb",
            DatabaseType::SingleStore => "singlestore",
            DatabaseType::Snowflake => "snowflake",
            DatabaseType::Redshift => "redshift",
            DatabaseType::BigQuery => "bigquery",
            DatabaseType::Databricks => "databricks",
            DatabaseType::Trino => "trino",
            DatabaseType::Presto => "presto",
            DatabaseType::Impala => "impala",
            DatabaseType::Hive => "hive",
            DatabaseType::Phoenix => "phoenix",
            DatabaseType::MongoDb => "mongodb",
            DatabaseType::Redis => "redis",
            DatabaseType::Cassandra => "cassandra",
            DatabaseType::ScyllaDb => "scylladb",
            DatabaseType::Neo4j => "neo4j",
            DatabaseType::CouchDb => "couchdb",
            DatabaseType::Couchbase => "couchbase",
            DatabaseType::Elasticsearch => "elasticsearch",
            DatabaseType::OpenSearch => "opensearch",
            DatabaseType::ArangoDb => "arangodb",
            DatabaseType::OrientDb => "orientdb",
            DatabaseType::DynamoDb => "dynamodb",
            DatabaseType::CosmosDb => "cosmosdb",
            DatabaseType::Firebase => "firebase",
            DatabaseType::Riak => "riak",
            DatabaseType::Memcached => "memcached",
            DatabaseType::Etcd => "etcd",
            DatabaseType::Consul => "consul",
            DatabaseType::HBase => "hbase",
            DatabaseType::Accumulo => "accumulo",
            DatabaseType::InfluxDb => "influxdb",
            DatabaseType::TimescaleDb => "timescaledb",
            DatabaseType::QuestDb => "questdb",
            DatabaseType::Prometheus => "prometheus",
            DatabaseType::VictoriaMetrics => "victoriametrics",
            DatabaseType::Supabase => "supabase",
            DatabaseType::PlanetScale => "planetscale",
            DatabaseType::Neon => "neon",
            DatabaseType::FaunaDb => "faunadb",
            DatabaseType::Airtable => "airtable",
            DatabaseType::Notion => "notion",
            DatabaseType::DuckDbEmbedded => "duckdb-emb",
            DatabaseType::SqliteEmbedded => "sqlite-emb",
            DatabaseType::RocksDb => "rocksdb",
            DatabaseType::Lmdb => "lmdb",
            DatabaseType::Sled => "sled",
            DatabaseType::Redb => "redb",
            DatabaseType::S3 => "s3",
            DatabaseType::Minio => "minio",
            DatabaseType::Kafka => "kafka",
            DatabaseType::RabbitMq => "rabbitmq",
            DatabaseType::Nats => "nats",
            DatabaseType::Pulsar => "pulsar",
            DatabaseType::Other => "other",
        }
    }
    /// Returns a human-readable English display name.
    pub const fn as_str(&self) -> &'static str {
        match self {
            DatabaseType::MySql => "MySQL",
            DatabaseType::MariaDb => "MariaDB",
            DatabaseType::PostgreSQL => "PostgreSQL",
            DatabaseType::SQLite => "SQLite",
            DatabaseType::SqlServer => "SQL Server",
            DatabaseType::Oracle => "Oracle",
            DatabaseType::Db2 => "IBM Db2",
            DatabaseType::ClickHouse => "ClickHouse",
            DatabaseType::DuckDb => "DuckDB",
            DatabaseType::Firebird => "Firebird",
            DatabaseType::H2 => "H2",
            DatabaseType::Derby => "Apache Derby",
            DatabaseType::Hsqldb => "HSQLDB",
            DatabaseType::Sybase => "Sybase",
            DatabaseType::Teradata => "Teradata",
            DatabaseType::Vertica => "Vertica",
            DatabaseType::Greenplum => "Greenplum",
            DatabaseType::CockroachDb => "CockroachDB",
            DatabaseType::TiDb => "TiDB",
            DatabaseType::YugabyteDb => "YugabyteDB",
            DatabaseType::SingleStore => "SingleStore",
            DatabaseType::Snowflake => "Snowflake",
            DatabaseType::Redshift => "Redshift",
            DatabaseType::BigQuery => "BigQuery",
            DatabaseType::Databricks => "Databricks",
            DatabaseType::Trino => "Trino",
            DatabaseType::Presto => "Presto",
            DatabaseType::Impala => "Apache Impala",
            DatabaseType::Hive => "Apache Hive",
            DatabaseType::Phoenix => "Apache Phoenix",
            DatabaseType::MongoDb => "MongoDB",
            DatabaseType::Redis => "Redis",
            DatabaseType::Cassandra => "Cassandra",
            DatabaseType::ScyllaDb => "ScyllaDB",
            DatabaseType::Neo4j => "Neo4j",
            DatabaseType::CouchDb => "CouchDB",
            DatabaseType::Couchbase => "Couchbase",
            DatabaseType::Elasticsearch => "Elasticsearch",
            DatabaseType::OpenSearch => "OpenSearch",
            DatabaseType::ArangoDb => "ArangoDB",
            DatabaseType::OrientDb => "OrientDB",
            DatabaseType::DynamoDb => "DynamoDB",
            DatabaseType::CosmosDb => "Azure Cosmos DB",
            DatabaseType::Firebase => "Firebase",
            DatabaseType::Riak => "Riak",
            DatabaseType::Memcached => "Memcached",
            DatabaseType::Etcd => "etcd",
            DatabaseType::Consul => "Consul",
            DatabaseType::HBase => "Apache HBase",
            DatabaseType::Accumulo => "Apache Accumulo",
            DatabaseType::InfluxDb => "InfluxDB",
            DatabaseType::TimescaleDb => "TimescaleDB",
            DatabaseType::QuestDb => "QuestDB",
            DatabaseType::Prometheus => "Prometheus",
            DatabaseType::VictoriaMetrics => "VictoriaMetrics",
            DatabaseType::Supabase => "Supabase",
            DatabaseType::PlanetScale => "PlanetScale",
            DatabaseType::Neon => "Neon",
            DatabaseType::FaunaDb => "FaunaDB",
            DatabaseType::Airtable => "Airtable",
            DatabaseType::Notion => "Notion",
            DatabaseType::DuckDbEmbedded => "DuckDB (Embedded)",
            DatabaseType::SqliteEmbedded => "SQLite (Embedded)",
            DatabaseType::RocksDb => "RocksDB",
            DatabaseType::Lmdb => "LMDB",
            DatabaseType::Sled => "Sled",
            DatabaseType::Redb => "Redb",
            DatabaseType::S3 => "Amazon S3",
            DatabaseType::Minio => "MinIO",
            DatabaseType::Kafka => "Apache Kafka",
            DatabaseType::RabbitMq => "RabbitMQ",
            DatabaseType::Nats => "NATS",
            DatabaseType::Pulsar => "Apache Pulsar",
            DatabaseType::Other => "Other",
        }
    }
    /// Returns the Chinese display name for this engine.
    pub const fn as_str_zh(&self) -> &'static str {
        match self {
            DatabaseType::MySql => "MySQL",
            DatabaseType::MariaDb => "MariaDB",
            DatabaseType::PostgreSQL => "PostgreSQL",
            DatabaseType::SQLite => "SQLite",
            DatabaseType::SqlServer => "SQL Server",
            DatabaseType::Oracle => "Oracle",
            DatabaseType::Db2 => "IBM Db2",
            DatabaseType::ClickHouse => "ClickHouse",
            DatabaseType::DuckDb => "DuckDB",
            DatabaseType::Firebird => "Firebird",
            DatabaseType::H2 => "H2",
            DatabaseType::Derby => "Apache Derby",
            DatabaseType::Hsqldb => "HSQLDB",
            DatabaseType::Sybase => "Sybase",
            DatabaseType::Teradata => "Teradata",
            DatabaseType::Vertica => "Vertica",
            DatabaseType::Greenplum => "Greenplum",
            DatabaseType::CockroachDb => "CockroachDB",
            DatabaseType::TiDb => "TiDB",
            DatabaseType::YugabyteDb => "YugabyteDB",
            DatabaseType::SingleStore => "SingleStore",
            DatabaseType::Snowflake => "Snowflake",
            DatabaseType::Redshift => "Redshift",
            DatabaseType::BigQuery => "BigQuery",
            DatabaseType::Databricks => "Databricks",
            DatabaseType::Trino => "Trino",
            DatabaseType::Presto => "Presto",
            DatabaseType::Impala => "Apache Impala",
            DatabaseType::Hive => "Apache Hive",
            DatabaseType::Phoenix => "Apache Phoenix",
            DatabaseType::MongoDb => "MongoDB",
            DatabaseType::Redis => "Redis",
            DatabaseType::Cassandra => "Cassandra",
            DatabaseType::ScyllaDb => "ScyllaDB",
            DatabaseType::Neo4j => "Neo4j",
            DatabaseType::CouchDb => "CouchDB",
            DatabaseType::Couchbase => "Couchbase",
            DatabaseType::Elasticsearch => "Elasticsearch",
            DatabaseType::OpenSearch => "OpenSearch",
            DatabaseType::ArangoDb => "ArangoDB",
            DatabaseType::OrientDb => "OrientDB",
            DatabaseType::DynamoDb => "DynamoDB",
            DatabaseType::CosmosDb => "Azure Cosmos DB",
            DatabaseType::Firebase => "Firebase",
            DatabaseType::Riak => "Riak",
            DatabaseType::Memcached => "Memcached",
            DatabaseType::Etcd => "etcd",
            DatabaseType::Consul => "Consul",
            DatabaseType::HBase => "Apache HBase",
            DatabaseType::Accumulo => "Apache Accumulo",
            DatabaseType::InfluxDb => "InfluxDB",
            DatabaseType::TimescaleDb => "TimescaleDB",
            DatabaseType::QuestDb => "QuestDB",
            DatabaseType::Prometheus => "Prometheus",
            DatabaseType::VictoriaMetrics => "VictoriaMetrics",
            DatabaseType::Supabase => "Supabase",
            DatabaseType::PlanetScale => "PlanetScale",
            DatabaseType::Neon => "Neon",
            DatabaseType::FaunaDb => "FaunaDB",
            DatabaseType::Airtable => "Airtable",
            DatabaseType::Notion => "Notion",
            DatabaseType::DuckDbEmbedded => "DuckDB（嵌入式）",
            DatabaseType::SqliteEmbedded => "SQLite（嵌入式）",
            DatabaseType::RocksDb => "RocksDB",
            DatabaseType::Lmdb => "LMDB",
            DatabaseType::Sled => "Sled",
            DatabaseType::Redb => "Redb",
            DatabaseType::S3 => "Amazon S3",
            DatabaseType::Minio => "MinIO",
            DatabaseType::Kafka => "Apache Kafka",
            DatabaseType::RabbitMq => "RabbitMQ",
            DatabaseType::Nats => "NATS",
            DatabaseType::Pulsar => "Apache Pulsar",
            DatabaseType::Other => "其他",
        }
    }
    /// Parses a database type from its canonical lowercase key.
    pub fn from_key(key: &str) -> Self {
        match key {
            "mysql" => DatabaseType::MySql,
            "mariadb" => DatabaseType::MariaDb,
            "postgresql" => DatabaseType::PostgreSQL,
            "sqlite" => DatabaseType::SQLite,
            "sqlserver" => DatabaseType::SqlServer,
            "oracle" => DatabaseType::Oracle,
            "db2" => DatabaseType::Db2,
            "clickhouse" => DatabaseType::ClickHouse,
            "duckdb" => DatabaseType::DuckDb,
            "firebird" => DatabaseType::Firebird,
            "h2" => DatabaseType::H2,
            "derby" => DatabaseType::Derby,
            "hsqldb" => DatabaseType::Hsqldb,
            "sybase" => DatabaseType::Sybase,
            "teradata" => DatabaseType::Teradata,
            "vertica" => DatabaseType::Vertica,
            "greenplum" => DatabaseType::Greenplum,
            "cockroachdb" => DatabaseType::CockroachDb,
            "tidb" => DatabaseType::TiDb,
            "yugabytedb" => DatabaseType::YugabyteDb,
            "singlestore" => DatabaseType::SingleStore,
            "snowflake" => DatabaseType::Snowflake,
            "redshift" => DatabaseType::Redshift,
            "bigquery" => DatabaseType::BigQuery,
            "databricks" => DatabaseType::Databricks,
            "trino" => DatabaseType::Trino,
            "presto" => DatabaseType::Presto,
            "impala" => DatabaseType::Impala,
            "hive" => DatabaseType::Hive,
            "phoenix" => DatabaseType::Phoenix,
            "mongodb" => DatabaseType::MongoDb,
            "redis" => DatabaseType::Redis,
            "cassandra" => DatabaseType::Cassandra,
            "scylladb" => DatabaseType::ScyllaDb,
            "neo4j" => DatabaseType::Neo4j,
            "couchdb" => DatabaseType::CouchDb,
            "couchbase" => DatabaseType::Couchbase,
            "elasticsearch" => DatabaseType::Elasticsearch,
            "opensearch" => DatabaseType::OpenSearch,
            "arangodb" => DatabaseType::ArangoDb,
            "orientdb" => DatabaseType::OrientDb,
            "dynamodb" => DatabaseType::DynamoDb,
            "cosmosdb" => DatabaseType::CosmosDb,
            "firebase" => DatabaseType::Firebase,
            "riak" => DatabaseType::Riak,
            "memcached" => DatabaseType::Memcached,
            "etcd" => DatabaseType::Etcd,
            "consul" => DatabaseType::Consul,
            "hbase" => DatabaseType::HBase,
            "accumulo" => DatabaseType::Accumulo,
            "influxdb" => DatabaseType::InfluxDb,
            "timescaledb" => DatabaseType::TimescaleDb,
            "questdb" => DatabaseType::QuestDb,
            "prometheus" => DatabaseType::Prometheus,
            "victoriametrics" => DatabaseType::VictoriaMetrics,
            "supabase" => DatabaseType::Supabase,
            "planetscale" => DatabaseType::PlanetScale,
            "neon" => DatabaseType::Neon,
            "faunadb" => DatabaseType::FaunaDb,
            "airtable" => DatabaseType::Airtable,
            "notion" => DatabaseType::Notion,
            "duckdb-emb" => DatabaseType::DuckDbEmbedded,
            "sqlite-emb" => DatabaseType::SqliteEmbedded,
            "rocksdb" => DatabaseType::RocksDb,
            "lmdb" => DatabaseType::Lmdb,
            "sled" => DatabaseType::Sled,
            "redb" => DatabaseType::Redb,
            "s3" => DatabaseType::S3,
            "minio" => DatabaseType::Minio,
            "kafka" => DatabaseType::Kafka,
            "rabbitmq" => DatabaseType::RabbitMq,
            "nats" => DatabaseType::Nats,
            "pulsar" => DatabaseType::Pulsar,
            _ => DatabaseType::Other,
        }
    }
    /// Returns every known [`DatabaseType`] variant.
    pub const fn all() -> &'static [DatabaseType] {
        &[
            DatabaseType::MySql,
            DatabaseType::MariaDb,
            DatabaseType::PostgreSQL,
            DatabaseType::SQLite,
            DatabaseType::SqlServer,
            DatabaseType::Oracle,
            DatabaseType::Db2,
            DatabaseType::ClickHouse,
            DatabaseType::DuckDb,
            DatabaseType::Firebird,
            DatabaseType::H2,
            DatabaseType::Derby,
            DatabaseType::Hsqldb,
            DatabaseType::Sybase,
            DatabaseType::Teradata,
            DatabaseType::Vertica,
            DatabaseType::Greenplum,
            DatabaseType::CockroachDb,
            DatabaseType::TiDb,
            DatabaseType::YugabyteDb,
            DatabaseType::SingleStore,
            DatabaseType::Snowflake,
            DatabaseType::Redshift,
            DatabaseType::BigQuery,
            DatabaseType::Databricks,
            DatabaseType::Trino,
            DatabaseType::Presto,
            DatabaseType::Impala,
            DatabaseType::Hive,
            DatabaseType::Phoenix,
            DatabaseType::MongoDb,
            DatabaseType::Redis,
            DatabaseType::Cassandra,
            DatabaseType::ScyllaDb,
            DatabaseType::Neo4j,
            DatabaseType::CouchDb,
            DatabaseType::Couchbase,
            DatabaseType::Elasticsearch,
            DatabaseType::OpenSearch,
            DatabaseType::ArangoDb,
            DatabaseType::OrientDb,
            DatabaseType::DynamoDb,
            DatabaseType::CosmosDb,
            DatabaseType::Firebase,
            DatabaseType::Riak,
            DatabaseType::Memcached,
            DatabaseType::Etcd,
            DatabaseType::Consul,
            DatabaseType::HBase,
            DatabaseType::Accumulo,
            DatabaseType::InfluxDb,
            DatabaseType::TimescaleDb,
            DatabaseType::QuestDb,
            DatabaseType::Prometheus,
            DatabaseType::VictoriaMetrics,
            DatabaseType::Supabase,
            DatabaseType::PlanetScale,
            DatabaseType::Neon,
            DatabaseType::FaunaDb,
            DatabaseType::Airtable,
            DatabaseType::Notion,
            DatabaseType::DuckDbEmbedded,
            DatabaseType::SqliteEmbedded,
            DatabaseType::RocksDb,
            DatabaseType::Lmdb,
            DatabaseType::Sled,
            DatabaseType::Redb,
            DatabaseType::S3,
            DatabaseType::Minio,
            DatabaseType::Kafka,
            DatabaseType::RabbitMq,
            DatabaseType::Nats,
            DatabaseType::Pulsar,
            DatabaseType::Other,
        ]
    }
}
impl fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
