//! Core database type definitions for dbuni.
use std::fmt;
/// The broad category a database belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DBUniCategory {
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
impl DBUniCategory {
    /// Returns the canonical lowercase key for this category.
    pub const fn as_key(&self) -> &'static str {
        match self {
            DBUniCategory::Sql => "sql",
            DBUniCategory::NoSql => "nosql",
            DBUniCategory::Cloud => "cloud",
            DBUniCategory::Embedded => "embedded",
            DBUniCategory::TimeSeries => "timeseries",
            DBUniCategory::Graph => "graph",
            DBUniCategory::Search => "search",
            DBUniCategory::KeyValue => "keyvalue",
            DBUniCategory::WideColumn => "widecolumn",
            DBUniCategory::ObjectStorage => "objectstorage",
            DBUniCategory::MessageQueue => "messagequeue",
            DBUniCategory::Other => "other",
        }
    }
    /// Returns a human-readable English label.
    pub const fn as_str(&self) -> &'static str {
        match self {
            DBUniCategory::Sql => "SQL",
            DBUniCategory::NoSql => "NoSQL",
            DBUniCategory::Cloud => "Cloud",
            DBUniCategory::Embedded => "Embedded",
            DBUniCategory::TimeSeries => "Time-Series",
            DBUniCategory::Graph => "Graph",
            DBUniCategory::Search => "Search",
            DBUniCategory::KeyValue => "Key-Value",
            DBUniCategory::WideColumn => "Wide-Column",
            DBUniCategory::ObjectStorage => "Object Storage",
            DBUniCategory::MessageQueue => "Message Queue",
            DBUniCategory::Other => "Other",
        }
    }
}
impl fmt::Display for DBUniCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
/// A concrete database engine supported by dbuni.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DBUniDatabase {
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
impl DBUniDatabase {
    /// Returns the broad category this engine belongs to.
    pub const fn category(&self) -> DBUniCategory {
        match self {
            // Relational / SQL
            DBUniDatabase::MySql
            | DBUniDatabase::MariaDb
            | DBUniDatabase::PostgreSQL
            | DBUniDatabase::SQLite
            | DBUniDatabase::SqlServer
            | DBUniDatabase::Oracle
            | DBUniDatabase::Db2
            | DBUniDatabase::ClickHouse
            | DBUniDatabase::DuckDb
            | DBUniDatabase::Firebird
            | DBUniDatabase::H2
            | DBUniDatabase::Derby
            | DBUniDatabase::Hsqldb
            | DBUniDatabase::Sybase
            | DBUniDatabase::Teradata
            | DBUniDatabase::Vertica
            | DBUniDatabase::Greenplum
            | DBUniDatabase::CockroachDb
            | DBUniDatabase::TiDb
            | DBUniDatabase::YugabyteDb
            | DBUniDatabase::SingleStore
            | DBUniDatabase::Trino
            | DBUniDatabase::Presto
            | DBUniDatabase::Impala
            | DBUniDatabase::Hive
            | DBUniDatabase::Phoenix => DBUniCategory::Sql,
            // Cloud / managed warehouses
            DBUniDatabase::Snowflake
            | DBUniDatabase::Redshift
            | DBUniDatabase::BigQuery
            | DBUniDatabase::Databricks => DBUniCategory::Cloud,
            // Document / key-value / wide-column / graph
            DBUniDatabase::MongoDb
            | DBUniDatabase::CouchDb
            | DBUniDatabase::Couchbase
            | DBUniDatabase::DynamoDb
            | DBUniDatabase::CosmosDb
            | DBUniDatabase::Firebase
            | DBUniDatabase::Riak
            | DBUniDatabase::FaunaDb
            | DBUniDatabase::Airtable
            | DBUniDatabase::Notion => DBUniCategory::NoSql,
            DBUniDatabase::Redis
            | DBUniDatabase::Memcached
            | DBUniDatabase::Etcd
            | DBUniDatabase::Consul => DBUniCategory::KeyValue,
            DBUniDatabase::Cassandra
            | DBUniDatabase::ScyllaDb
            | DBUniDatabase::HBase
            | DBUniDatabase::Accumulo => DBUniCategory::WideColumn,
            DBUniDatabase::Neo4j | DBUniDatabase::ArangoDb | DBUniDatabase::OrientDb => {
                DBUniCategory::Graph
            }
            DBUniDatabase::Elasticsearch | DBUniDatabase::OpenSearch => DBUniCategory::Search,
            DBUniDatabase::InfluxDb
            | DBUniDatabase::TimescaleDb
            | DBUniDatabase::QuestDb
            | DBUniDatabase::Prometheus
            | DBUniDatabase::VictoriaMetrics => DBUniCategory::TimeSeries,
            // Cloud databases (non-warehouse)
            DBUniDatabase::Supabase | DBUniDatabase::PlanetScale | DBUniDatabase::Neon => {
                DBUniCategory::Cloud
            }
            // Embedded
            DBUniDatabase::DuckDbEmbedded
            | DBUniDatabase::SqliteEmbedded
            | DBUniDatabase::RocksDb
            | DBUniDatabase::Lmdb
            | DBUniDatabase::Sled
            | DBUniDatabase::Redb => DBUniCategory::Embedded,
            // Object storage
            DBUniDatabase::S3 | DBUniDatabase::Minio => DBUniCategory::ObjectStorage,
            // Message queue / streaming
            DBUniDatabase::Kafka
            | DBUniDatabase::RabbitMq
            | DBUniDatabase::Nats
            | DBUniDatabase::Pulsar => DBUniCategory::MessageQueue,
            DBUniDatabase::Other => DBUniCategory::Other,
        }
    }
    /// Returns the canonical lowercase key for this engine.
    pub const fn as_key(&self) -> &'static str {
        match self {
            DBUniDatabase::MySql => "mysql",
            DBUniDatabase::MariaDb => "mariadb",
            DBUniDatabase::PostgreSQL => "postgresql",
            DBUniDatabase::SQLite => "sqlite",
            DBUniDatabase::SqlServer => "sqlserver",
            DBUniDatabase::Oracle => "oracle",
            DBUniDatabase::Db2 => "db2",
            DBUniDatabase::ClickHouse => "clickhouse",
            DBUniDatabase::DuckDb => "duckdb",
            DBUniDatabase::Firebird => "firebird",
            DBUniDatabase::H2 => "h2",
            DBUniDatabase::Derby => "derby",
            DBUniDatabase::Hsqldb => "hsqldb",
            DBUniDatabase::Sybase => "sybase",
            DBUniDatabase::Teradata => "teradata",
            DBUniDatabase::Vertica => "vertica",
            DBUniDatabase::Greenplum => "greenplum",
            DBUniDatabase::CockroachDb => "cockroachdb",
            DBUniDatabase::TiDb => "tidb",
            DBUniDatabase::YugabyteDb => "yugabytedb",
            DBUniDatabase::SingleStore => "singlestore",
            DBUniDatabase::Snowflake => "snowflake",
            DBUniDatabase::Redshift => "redshift",
            DBUniDatabase::BigQuery => "bigquery",
            DBUniDatabase::Databricks => "databricks",
            DBUniDatabase::Trino => "trino",
            DBUniDatabase::Presto => "presto",
            DBUniDatabase::Impala => "impala",
            DBUniDatabase::Hive => "hive",
            DBUniDatabase::Phoenix => "phoenix",
            DBUniDatabase::MongoDb => "mongodb",
            DBUniDatabase::Redis => "redis",
            DBUniDatabase::Cassandra => "cassandra",
            DBUniDatabase::ScyllaDb => "scylladb",
            DBUniDatabase::Neo4j => "neo4j",
            DBUniDatabase::CouchDb => "couchdb",
            DBUniDatabase::Couchbase => "couchbase",
            DBUniDatabase::Elasticsearch => "elasticsearch",
            DBUniDatabase::OpenSearch => "opensearch",
            DBUniDatabase::ArangoDb => "arangodb",
            DBUniDatabase::OrientDb => "orientdb",
            DBUniDatabase::DynamoDb => "dynamodb",
            DBUniDatabase::CosmosDb => "cosmosdb",
            DBUniDatabase::Firebase => "firebase",
            DBUniDatabase::Riak => "riak",
            DBUniDatabase::Memcached => "memcached",
            DBUniDatabase::Etcd => "etcd",
            DBUniDatabase::Consul => "consul",
            DBUniDatabase::HBase => "hbase",
            DBUniDatabase::Accumulo => "accumulo",
            DBUniDatabase::InfluxDb => "influxdb",
            DBUniDatabase::TimescaleDb => "timescaledb",
            DBUniDatabase::QuestDb => "questdb",
            DBUniDatabase::Prometheus => "prometheus",
            DBUniDatabase::VictoriaMetrics => "victoriametrics",
            DBUniDatabase::Supabase => "supabase",
            DBUniDatabase::PlanetScale => "planetscale",
            DBUniDatabase::Neon => "neon",
            DBUniDatabase::FaunaDb => "faunadb",
            DBUniDatabase::Airtable => "airtable",
            DBUniDatabase::Notion => "notion",
            DBUniDatabase::DuckDbEmbedded => "duckdb-emb",
            DBUniDatabase::SqliteEmbedded => "sqlite-emb",
            DBUniDatabase::RocksDb => "rocksdb",
            DBUniDatabase::Lmdb => "lmdb",
            DBUniDatabase::Sled => "sled",
            DBUniDatabase::Redb => "redb",
            DBUniDatabase::S3 => "s3",
            DBUniDatabase::Minio => "minio",
            DBUniDatabase::Kafka => "kafka",
            DBUniDatabase::RabbitMq => "rabbitmq",
            DBUniDatabase::Nats => "nats",
            DBUniDatabase::Pulsar => "pulsar",
            DBUniDatabase::Other => "other",
        }
    }
    /// Returns a human-readable English display name.
    pub const fn as_str(&self) -> &'static str {
        match self {
            DBUniDatabase::MySql => "MySQL",
            DBUniDatabase::MariaDb => "MariaDB",
            DBUniDatabase::PostgreSQL => "PostgreSQL",
            DBUniDatabase::SQLite => "SQLite",
            DBUniDatabase::SqlServer => "SQL Server",
            DBUniDatabase::Oracle => "Oracle",
            DBUniDatabase::Db2 => "IBM Db2",
            DBUniDatabase::ClickHouse => "ClickHouse",
            DBUniDatabase::DuckDb => "DuckDB",
            DBUniDatabase::Firebird => "Firebird",
            DBUniDatabase::H2 => "H2",
            DBUniDatabase::Derby => "Apache Derby",
            DBUniDatabase::Hsqldb => "HSQLDB",
            DBUniDatabase::Sybase => "Sybase",
            DBUniDatabase::Teradata => "Teradata",
            DBUniDatabase::Vertica => "Vertica",
            DBUniDatabase::Greenplum => "Greenplum",
            DBUniDatabase::CockroachDb => "CockroachDB",
            DBUniDatabase::TiDb => "TiDB",
            DBUniDatabase::YugabyteDb => "YugabyteDB",
            DBUniDatabase::SingleStore => "SingleStore",
            DBUniDatabase::Snowflake => "Snowflake",
            DBUniDatabase::Redshift => "Redshift",
            DBUniDatabase::BigQuery => "BigQuery",
            DBUniDatabase::Databricks => "Databricks",
            DBUniDatabase::Trino => "Trino",
            DBUniDatabase::Presto => "Presto",
            DBUniDatabase::Impala => "Apache Impala",
            DBUniDatabase::Hive => "Apache Hive",
            DBUniDatabase::Phoenix => "Apache Phoenix",
            DBUniDatabase::MongoDb => "MongoDB",
            DBUniDatabase::Redis => "Redis",
            DBUniDatabase::Cassandra => "Cassandra",
            DBUniDatabase::ScyllaDb => "ScyllaDB",
            DBUniDatabase::Neo4j => "Neo4j",
            DBUniDatabase::CouchDb => "CouchDB",
            DBUniDatabase::Couchbase => "Couchbase",
            DBUniDatabase::Elasticsearch => "Elasticsearch",
            DBUniDatabase::OpenSearch => "OpenSearch",
            DBUniDatabase::ArangoDb => "ArangoDB",
            DBUniDatabase::OrientDb => "OrientDB",
            DBUniDatabase::DynamoDb => "DynamoDB",
            DBUniDatabase::CosmosDb => "Azure Cosmos DB",
            DBUniDatabase::Firebase => "Firebase",
            DBUniDatabase::Riak => "Riak",
            DBUniDatabase::Memcached => "Memcached",
            DBUniDatabase::Etcd => "etcd",
            DBUniDatabase::Consul => "Consul",
            DBUniDatabase::HBase => "Apache HBase",
            DBUniDatabase::Accumulo => "Apache Accumulo",
            DBUniDatabase::InfluxDb => "InfluxDB",
            DBUniDatabase::TimescaleDb => "TimescaleDB",
            DBUniDatabase::QuestDb => "QuestDB",
            DBUniDatabase::Prometheus => "Prometheus",
            DBUniDatabase::VictoriaMetrics => "VictoriaMetrics",
            DBUniDatabase::Supabase => "Supabase",
            DBUniDatabase::PlanetScale => "PlanetScale",
            DBUniDatabase::Neon => "Neon",
            DBUniDatabase::FaunaDb => "FaunaDB",
            DBUniDatabase::Airtable => "Airtable",
            DBUniDatabase::Notion => "Notion",
            DBUniDatabase::DuckDbEmbedded => "DuckDB (Embedded)",
            DBUniDatabase::SqliteEmbedded => "SQLite (Embedded)",
            DBUniDatabase::RocksDb => "RocksDB",
            DBUniDatabase::Lmdb => "LMDB",
            DBUniDatabase::Sled => "Sled",
            DBUniDatabase::Redb => "Redb",
            DBUniDatabase::S3 => "Amazon S3",
            DBUniDatabase::Minio => "MinIO",
            DBUniDatabase::Kafka => "Apache Kafka",
            DBUniDatabase::RabbitMq => "RabbitMQ",
            DBUniDatabase::Nats => "NATS",
            DBUniDatabase::Pulsar => "Apache Pulsar",
            DBUniDatabase::Other => "Other",
        }
    }
    /// Returns the Chinese display name for this engine.
    pub const fn as_str_zh(&self) -> &'static str {
        match self {
            DBUniDatabase::MySql => "MySQL",
            DBUniDatabase::MariaDb => "MariaDB",
            DBUniDatabase::PostgreSQL => "PostgreSQL",
            DBUniDatabase::SQLite => "SQLite",
            DBUniDatabase::SqlServer => "SQL Server",
            DBUniDatabase::Oracle => "Oracle",
            DBUniDatabase::Db2 => "IBM Db2",
            DBUniDatabase::ClickHouse => "ClickHouse",
            DBUniDatabase::DuckDb => "DuckDB",
            DBUniDatabase::Firebird => "Firebird",
            DBUniDatabase::H2 => "H2",
            DBUniDatabase::Derby => "Apache Derby",
            DBUniDatabase::Hsqldb => "HSQLDB",
            DBUniDatabase::Sybase => "Sybase",
            DBUniDatabase::Teradata => "Teradata",
            DBUniDatabase::Vertica => "Vertica",
            DBUniDatabase::Greenplum => "Greenplum",
            DBUniDatabase::CockroachDb => "CockroachDB",
            DBUniDatabase::TiDb => "TiDB",
            DBUniDatabase::YugabyteDb => "YugabyteDB",
            DBUniDatabase::SingleStore => "SingleStore",
            DBUniDatabase::Snowflake => "Snowflake",
            DBUniDatabase::Redshift => "Redshift",
            DBUniDatabase::BigQuery => "BigQuery",
            DBUniDatabase::Databricks => "Databricks",
            DBUniDatabase::Trino => "Trino",
            DBUniDatabase::Presto => "Presto",
            DBUniDatabase::Impala => "Apache Impala",
            DBUniDatabase::Hive => "Apache Hive",
            DBUniDatabase::Phoenix => "Apache Phoenix",
            DBUniDatabase::MongoDb => "MongoDB",
            DBUniDatabase::Redis => "Redis",
            DBUniDatabase::Cassandra => "Cassandra",
            DBUniDatabase::ScyllaDb => "ScyllaDB",
            DBUniDatabase::Neo4j => "Neo4j",
            DBUniDatabase::CouchDb => "CouchDB",
            DBUniDatabase::Couchbase => "Couchbase",
            DBUniDatabase::Elasticsearch => "Elasticsearch",
            DBUniDatabase::OpenSearch => "OpenSearch",
            DBUniDatabase::ArangoDb => "ArangoDB",
            DBUniDatabase::OrientDb => "OrientDB",
            DBUniDatabase::DynamoDb => "DynamoDB",
            DBUniDatabase::CosmosDb => "Azure Cosmos DB",
            DBUniDatabase::Firebase => "Firebase",
            DBUniDatabase::Riak => "Riak",
            DBUniDatabase::Memcached => "Memcached",
            DBUniDatabase::Etcd => "etcd",
            DBUniDatabase::Consul => "Consul",
            DBUniDatabase::HBase => "Apache HBase",
            DBUniDatabase::Accumulo => "Apache Accumulo",
            DBUniDatabase::InfluxDb => "InfluxDB",
            DBUniDatabase::TimescaleDb => "TimescaleDB",
            DBUniDatabase::QuestDb => "QuestDB",
            DBUniDatabase::Prometheus => "Prometheus",
            DBUniDatabase::VictoriaMetrics => "VictoriaMetrics",
            DBUniDatabase::Supabase => "Supabase",
            DBUniDatabase::PlanetScale => "PlanetScale",
            DBUniDatabase::Neon => "Neon",
            DBUniDatabase::FaunaDb => "FaunaDB",
            DBUniDatabase::Airtable => "Airtable",
            DBUniDatabase::Notion => "Notion",
            DBUniDatabase::DuckDbEmbedded => "DuckDB（嵌入式）",
            DBUniDatabase::SqliteEmbedded => "SQLite（嵌入式）",
            DBUniDatabase::RocksDb => "RocksDB",
            DBUniDatabase::Lmdb => "LMDB",
            DBUniDatabase::Sled => "Sled",
            DBUniDatabase::Redb => "Redb",
            DBUniDatabase::S3 => "Amazon S3",
            DBUniDatabase::Minio => "MinIO",
            DBUniDatabase::Kafka => "Apache Kafka",
            DBUniDatabase::RabbitMq => "RabbitMQ",
            DBUniDatabase::Nats => "NATS",
            DBUniDatabase::Pulsar => "Apache Pulsar",
            DBUniDatabase::Other => "其他",
        }
    }
    /// Parses a database type from its canonical lowercase key.
    pub fn from_key(key: &str) -> Self {
        match key {
            "mysql" => DBUniDatabase::MySql,
            "mariadb" => DBUniDatabase::MariaDb,
            "postgresql" => DBUniDatabase::PostgreSQL,
            "sqlite" => DBUniDatabase::SQLite,
            "sqlserver" => DBUniDatabase::SqlServer,
            "oracle" => DBUniDatabase::Oracle,
            "db2" => DBUniDatabase::Db2,
            "clickhouse" => DBUniDatabase::ClickHouse,
            "duckdb" => DBUniDatabase::DuckDb,
            "firebird" => DBUniDatabase::Firebird,
            "h2" => DBUniDatabase::H2,
            "derby" => DBUniDatabase::Derby,
            "hsqldb" => DBUniDatabase::Hsqldb,
            "sybase" => DBUniDatabase::Sybase,
            "teradata" => DBUniDatabase::Teradata,
            "vertica" => DBUniDatabase::Vertica,
            "greenplum" => DBUniDatabase::Greenplum,
            "cockroachdb" => DBUniDatabase::CockroachDb,
            "tidb" => DBUniDatabase::TiDb,
            "yugabytedb" => DBUniDatabase::YugabyteDb,
            "singlestore" => DBUniDatabase::SingleStore,
            "snowflake" => DBUniDatabase::Snowflake,
            "redshift" => DBUniDatabase::Redshift,
            "bigquery" => DBUniDatabase::BigQuery,
            "databricks" => DBUniDatabase::Databricks,
            "trino" => DBUniDatabase::Trino,
            "presto" => DBUniDatabase::Presto,
            "impala" => DBUniDatabase::Impala,
            "hive" => DBUniDatabase::Hive,
            "phoenix" => DBUniDatabase::Phoenix,
            "mongodb" => DBUniDatabase::MongoDb,
            "redis" => DBUniDatabase::Redis,
            "cassandra" => DBUniDatabase::Cassandra,
            "scylladb" => DBUniDatabase::ScyllaDb,
            "neo4j" => DBUniDatabase::Neo4j,
            "couchdb" => DBUniDatabase::CouchDb,
            "couchbase" => DBUniDatabase::Couchbase,
            "elasticsearch" => DBUniDatabase::Elasticsearch,
            "opensearch" => DBUniDatabase::OpenSearch,
            "arangodb" => DBUniDatabase::ArangoDb,
            "orientdb" => DBUniDatabase::OrientDb,
            "dynamodb" => DBUniDatabase::DynamoDb,
            "cosmosdb" => DBUniDatabase::CosmosDb,
            "firebase" => DBUniDatabase::Firebase,
            "riak" => DBUniDatabase::Riak,
            "memcached" => DBUniDatabase::Memcached,
            "etcd" => DBUniDatabase::Etcd,
            "consul" => DBUniDatabase::Consul,
            "hbase" => DBUniDatabase::HBase,
            "accumulo" => DBUniDatabase::Accumulo,
            "influxdb" => DBUniDatabase::InfluxDb,
            "timescaledb" => DBUniDatabase::TimescaleDb,
            "questdb" => DBUniDatabase::QuestDb,
            "prometheus" => DBUniDatabase::Prometheus,
            "victoriametrics" => DBUniDatabase::VictoriaMetrics,
            "supabase" => DBUniDatabase::Supabase,
            "planetscale" => DBUniDatabase::PlanetScale,
            "neon" => DBUniDatabase::Neon,
            "faunadb" => DBUniDatabase::FaunaDb,
            "airtable" => DBUniDatabase::Airtable,
            "notion" => DBUniDatabase::Notion,
            "duckdb-emb" => DBUniDatabase::DuckDbEmbedded,
            "sqlite-emb" => DBUniDatabase::SqliteEmbedded,
            "rocksdb" => DBUniDatabase::RocksDb,
            "lmdb" => DBUniDatabase::Lmdb,
            "sled" => DBUniDatabase::Sled,
            "redb" => DBUniDatabase::Redb,
            "s3" => DBUniDatabase::S3,
            "minio" => DBUniDatabase::Minio,
            "kafka" => DBUniDatabase::Kafka,
            "rabbitmq" => DBUniDatabase::RabbitMq,
            "nats" => DBUniDatabase::Nats,
            "pulsar" => DBUniDatabase::Pulsar,
            _ => DBUniDatabase::Other,
        }
    }
    /// Returns every known [`DBUniDatabase`] variant.
    pub const fn all() -> &'static [DBUniDatabase] {
        &[
            DBUniDatabase::MySql,
            DBUniDatabase::MariaDb,
            DBUniDatabase::PostgreSQL,
            DBUniDatabase::SQLite,
            DBUniDatabase::SqlServer,
            DBUniDatabase::Oracle,
            DBUniDatabase::Db2,
            DBUniDatabase::ClickHouse,
            DBUniDatabase::DuckDb,
            DBUniDatabase::Firebird,
            DBUniDatabase::H2,
            DBUniDatabase::Derby,
            DBUniDatabase::Hsqldb,
            DBUniDatabase::Sybase,
            DBUniDatabase::Teradata,
            DBUniDatabase::Vertica,
            DBUniDatabase::Greenplum,
            DBUniDatabase::CockroachDb,
            DBUniDatabase::TiDb,
            DBUniDatabase::YugabyteDb,
            DBUniDatabase::SingleStore,
            DBUniDatabase::Snowflake,
            DBUniDatabase::Redshift,
            DBUniDatabase::BigQuery,
            DBUniDatabase::Databricks,
            DBUniDatabase::Trino,
            DBUniDatabase::Presto,
            DBUniDatabase::Impala,
            DBUniDatabase::Hive,
            DBUniDatabase::Phoenix,
            DBUniDatabase::MongoDb,
            DBUniDatabase::Redis,
            DBUniDatabase::Cassandra,
            DBUniDatabase::ScyllaDb,
            DBUniDatabase::Neo4j,
            DBUniDatabase::CouchDb,
            DBUniDatabase::Couchbase,
            DBUniDatabase::Elasticsearch,
            DBUniDatabase::OpenSearch,
            DBUniDatabase::ArangoDb,
            DBUniDatabase::OrientDb,
            DBUniDatabase::DynamoDb,
            DBUniDatabase::CosmosDb,
            DBUniDatabase::Firebase,
            DBUniDatabase::Riak,
            DBUniDatabase::Memcached,
            DBUniDatabase::Etcd,
            DBUniDatabase::Consul,
            DBUniDatabase::HBase,
            DBUniDatabase::Accumulo,
            DBUniDatabase::InfluxDb,
            DBUniDatabase::TimescaleDb,
            DBUniDatabase::QuestDb,
            DBUniDatabase::Prometheus,
            DBUniDatabase::VictoriaMetrics,
            DBUniDatabase::Supabase,
            DBUniDatabase::PlanetScale,
            DBUniDatabase::Neon,
            DBUniDatabase::FaunaDb,
            DBUniDatabase::Airtable,
            DBUniDatabase::Notion,
            DBUniDatabase::DuckDbEmbedded,
            DBUniDatabase::SqliteEmbedded,
            DBUniDatabase::RocksDb,
            DBUniDatabase::Lmdb,
            DBUniDatabase::Sled,
            DBUniDatabase::Redb,
            DBUniDatabase::S3,
            DBUniDatabase::Minio,
            DBUniDatabase::Kafka,
            DBUniDatabase::RabbitMq,
            DBUniDatabase::Nats,
            DBUniDatabase::Pulsar,
            DBUniDatabase::Other,
        ]
    }
}
impl fmt::Display for DBUniDatabase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
/// The unified response returned by every dbuni operation.
#[derive(Debug, Clone, PartialEq)]
pub struct DBUniResult {
    /// The database engine that produced this result.
    pub database: DBUniDatabase,
    /// A flat tag describing the shape of the payload.
    pub kind: DBUniResponseKind,
    /// Elapsed time of the operation, in milliseconds.
    pub elapsed_ms: u64,
    /// Number of rows or elements affected, when it can be computed.
    pub affected: Option<u64>,
    /// The actual data payload.
    pub payload: DBUniPayload,
}
impl DBUniResult {
    /// Creates a new result, deriving `kind` from the payload.
    pub fn new(
        database: DBUniDatabase,
        elapsed_ms: u64,
        affected: Option<u64>,
        payload: DBUniPayload,
    ) -> Self {
        let kind = payload.kind();
        Self {
            database,
            kind,
            elapsed_ms,
            affected,
            payload,
        }
    }
    /// Returns `true` if the payload carries a two-dimensional result set.
    pub const fn is_rows(&self) -> bool {
        matches!(self.kind, DBUniResponseKind::Rows)
    }
    /// Returns `true` if the payload carries an affected-row count.
    pub const fn is_affected(&self) -> bool {
        matches!(self.kind, DBUniResponseKind::Affected)
    }
    /// Returns the payload as a result set, if it is one.
    pub fn as_result_set(&self) -> Option<&DBUniResultSet> {
        match &self.payload {
            DBUniPayload::Rows(rs) => Some(rs),
            _ => None,
        }
    }
}
/// A flat tag describing the shape of a [`DBUniResult`] payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DBUniResponseKind {
    /// A two-dimensional result set (SQL SELECT).
    Rows,
    /// An affected-row count (SQL INSERT/UPDATE/DELETE).
    Affected,
    /// A single scalar value (Redis INCR, SQL count(*)).
    Scalar,
    /// A one-dimensional array (Redis LRANGE, MGET).
    Array,
    /// A set of key-value pairs (Redis HGETALL).
    Map,
    /// A set of documents (MongoDB, CouchDB).
    Document,
    /// A graph result (Neo4j nodes/edges/paths).
    Graph,
    /// Explicit success with no data (Redis SET returns +OK).
    Ok,
    /// No data returned at all.
    Empty,
}
impl DBUniResponseKind {
    /// Returns the canonical lowercase key for this response kind.
    pub const fn as_key(&self) -> &'static str {
        match self {
            DBUniResponseKind::Rows => "rows",
            DBUniResponseKind::Affected => "affected",
            DBUniResponseKind::Scalar => "scalar",
            DBUniResponseKind::Array => "array",
            DBUniResponseKind::Map => "map",
            DBUniResponseKind::Document => "document",
            DBUniResponseKind::Graph => "graph",
            DBUniResponseKind::Ok => "ok",
            DBUniResponseKind::Empty => "empty",
        }
    }
    /// Returns a human-readable English label.
    pub const fn as_str(&self) -> &'static str {
        match self {
            DBUniResponseKind::Rows => "Rows",
            DBUniResponseKind::Affected => "Affected",
            DBUniResponseKind::Scalar => "Scalar",
            DBUniResponseKind::Array => "Array",
            DBUniResponseKind::Map => "Map",
            DBUniResponseKind::Document => "Document",
            DBUniResponseKind::Graph => "Graph",
            DBUniResponseKind::Ok => "Ok",
            DBUniResponseKind::Empty => "Empty",
        }
    }
}
impl fmt::Display for DBUniResponseKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
/// The polymorphic payload carried by a [`DBUniResult`].
#[derive(Debug, Clone, PartialEq)]
pub enum DBUniPayload {
    /// A two-dimensional result set.
    Rows(DBUniResultSet),
    /// An affected-row or affected-element count.
    Affected(u64),
    /// A single scalar value.
    Scalar(DBUniValue),
    /// A one-dimensional array of values.
    Array(Vec<DBUniValue>),
    /// A set of key-value pairs.
    Map(Vec<(DBUniValue, DBUniValue)>),
    /// A set of documents.
    Document(Vec<DBUniValue>),
    /// A graph result.
    Graph(DBUniGraphResult),
    /// Explicit success with no data.
    Ok,
    /// No data.
    Empty,
}
impl DBUniPayload {
    /// Returns the matching [`DBUniResponseKind`] for this payload.
    pub const fn kind(&self) -> DBUniResponseKind {
        match self {
            DBUniPayload::Rows(_) => DBUniResponseKind::Rows,
            DBUniPayload::Affected(_) => DBUniResponseKind::Affected,
            DBUniPayload::Scalar(_) => DBUniResponseKind::Scalar,
            DBUniPayload::Array(_) => DBUniResponseKind::Array,
            DBUniPayload::Map(_) => DBUniResponseKind::Map,
            DBUniPayload::Document(_) => DBUniResponseKind::Document,
            DBUniPayload::Graph(_) => DBUniResponseKind::Graph,
            DBUniPayload::Ok => DBUniResponseKind::Ok,
            DBUniPayload::Empty => DBUniResponseKind::Empty,
        }
    }
}
/// A two-dimensional result set, used by SQL-style databases.
#[derive(Debug, Clone, PartialEq)]
pub struct DBUniResultSet {
    /// Column metadata, in column order.
    pub columns: Vec<DBUniColumn>,
    /// Row data. Each row is aligned with `columns` by index.
    pub rows: Vec<Vec<DBUniValue>>,
}
impl DBUniResultSet {
    /// Creates a new, empty result set.
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }
    /// Returns the number of rows in this result set.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
    /// Returns the number of columns in this result set.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }
}
impl Default for DBUniResultSet {
    fn default() -> Self {
        Self::new()
    }
}
/// Column metadata for a [`DBUniResultSet`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DBUniColumn {
    /// The column name.
    pub name: String,
    /// The unified column type.
    pub data_type: DBUniDataType,
}
impl DBUniColumn {
    /// Creates a new column descriptor.
    pub fn new(name: impl Into<String>, data_type: DBUniDataType) -> Self {
        Self {
            name: name.into(),
            data_type,
        }
    }
}
/// A unified scalar value.
#[derive(Debug, Clone, PartialEq)]
pub enum DBUniValue {
    /// Null or missing value.
    Null,
    /// Boolean value.
    Bool(bool),
    /// Signed integer.
    Int(i64),
    /// Unsigned integer.
    UInt(u64),
    /// Floating-point value.
    Float(f64),
    /// UTF-8 text.
    Text(String),
    /// Raw bytes.
    Bytes(Vec<u8>),
    /// Timestamp in milliseconds since the Unix epoch.
    Timestamp(i64),
    /// JSON text, kept as-is.
    Json(String),
    /// Nested array.
    Array(Vec<DBUniValue>),
    /// Nested key-value pairs.
    Map(Vec<(DBUniValue, DBUniValue)>),
}
impl DBUniValue {
    /// Returns `true` if this value is `Null`.
    pub const fn is_null(&self) -> bool {
        matches!(self, DBUniValue::Null)
    }
    /// Returns the matching [`DBUniDataType`] for this value.
    pub const fn data_type(&self) -> DBUniDataType {
        match self {
            DBUniValue::Null => DBUniDataType::Null,
            DBUniValue::Bool(_) => DBUniDataType::Bool,
            DBUniValue::Int(_) => DBUniDataType::Int,
            DBUniValue::UInt(_) => DBUniDataType::UInt,
            DBUniValue::Float(_) => DBUniDataType::Float,
            DBUniValue::Text(_) => DBUniDataType::Text,
            DBUniValue::Bytes(_) => DBUniDataType::Bytes,
            DBUniValue::Timestamp(_) => DBUniDataType::Timestamp,
            DBUniValue::Json(_) => DBUniDataType::Json,
            DBUniValue::Array(_) => DBUniDataType::Array,
            DBUniValue::Map(_) => DBUniDataType::Map,
        }
    }
}
/// A unified column type, coarser than [`DBUniValue`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DBUniDataType {
    /// Null or unknown at the type level.
    Null,
    /// Boolean.
    Bool,
    /// Signed integer.
    Int,
    /// Unsigned integer.
    UInt,
    /// Floating-point.
    Float,
    /// UTF-8 text.
    Text,
    /// Raw bytes.
    Bytes,
    /// Timestamp.
    Timestamp,
    /// JSON.
    Json,
    /// Array.
    Array,
    /// Map or object.
    Map,
    /// Type could not be determined.
    Unknown,
}
impl DBUniDataType {
    /// Returns the canonical lowercase key for this type.
    pub const fn as_key(&self) -> &'static str {
        match self {
            DBUniDataType::Null => "null",
            DBUniDataType::Bool => "bool",
            DBUniDataType::Int => "int",
            DBUniDataType::UInt => "uint",
            DBUniDataType::Float => "float",
            DBUniDataType::Text => "text",
            DBUniDataType::Bytes => "bytes",
            DBUniDataType::Timestamp => "timestamp",
            DBUniDataType::Json => "json",
            DBUniDataType::Array => "array",
            DBUniDataType::Map => "map",
            DBUniDataType::Unknown => "unknown",
        }
    }
    /// Returns a human-readable English label.
    pub const fn as_str(&self) -> &'static str {
        match self {
            DBUniDataType::Null => "Null",
            DBUniDataType::Bool => "Bool",
            DBUniDataType::Int => "Int",
            DBUniDataType::UInt => "UInt",
            DBUniDataType::Float => "Float",
            DBUniDataType::Text => "Text",
            DBUniDataType::Bytes => "Bytes",
            DBUniDataType::Timestamp => "Timestamp",
            DBUniDataType::Json => "Json",
            DBUniDataType::Array => "Array",
            DBUniDataType::Map => "Map",
            DBUniDataType::Unknown => "Unknown",
        }
    }
}
impl fmt::Display for DBUniDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
/// A graph result, used by graph databases such as Neo4j and ArangoDB.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DBUniGraphResult {
    /// Nodes in the result.
    pub nodes: Vec<DBUniValue>,
    /// Edges in the result.
    pub edges: Vec<DBUniValue>,
    /// Paths in the result.
    pub paths: Vec<DBUniValue>,
}
/// The unified error type for every dbuni operation.
#[derive(Debug, Clone, PartialEq)]
pub enum DBUniError {
    /// The connection could not be established.
    ConnectionFailed(String),
    /// The connection was lost while in use.
    ConnectionLost(String),
    /// Authentication failed (bad user, password, or token).
    AuthenticationFailed(String),
    /// The connection or operation timed out.
    Timeout(String),
    /// The SQL or command text is malformed.
    SyntaxError {
        /// The offending statement or command.
        statement: String,
        /// A human-readable description of the problem.
        message: String,
    },
    /// A referenced table, column, key, or collection does not exist.
    NotFound {
        /// The kind of object that was not found.
        object: String,
        /// The object name.
        name: String,
    },
    /// A unique or primary-key constraint was violated.
    UniqueViolation(String),
    /// A foreign-key constraint was violated.
    ForeignKeyViolation(String),
    /// A not-null constraint was violated.
    NotNullViolation(String),
    /// A check constraint was violated.
    CheckViolation(String),
    /// A deadlock or lock-wait timeout occurred.
    Deadlock(String),
    /// A transaction was aborted or rolled back.
    TransactionAborted(String),
    /// The requested type conversion failed.
    TypeMismatch {
        /// The source type.
        from: String,
        /// The target type.
        to: String,
    },
    /// The provided value is out of range for the target type.
    OutOfRange(String),
    /// A query returned no rows where at least one was expected.
    NoRowsReturned,
    /// A query returned more rows than expected.
    TooManyRowsReturned {
        /// The number of rows actually returned.
        actual: usize,
    },
    /// The operation is not supported by this driver.
    Unsupported(String),
    /// The feature has not been implemented yet.
    NotImplemented(String),
    /// A parameter was not bound or was bound incorrectly.
    ParameterError(String),
    /// A driver-specific error that could not be mapped.
    Driver {
        /// The database engine that produced the error.
        database: DBUniDatabase,
        /// The driver's native error code, if any.
        code: Option<String>,
        /// The driver's native error message.
        message: String,
    },
    /// A serialization or deserialization failure.
    Serialization(String),
    /// An I/O error surfaced from the underlying transport.
    Io(String),
    /// The operation was cancelled by the caller.
    Cancelled,
    /// A catch-all for errors that do not fit any other variant.
    Other(String),
}
impl DBUniError {
    /// Returns the canonical lowercase key for this error kind.
    pub const fn as_key(&self) -> &'static str {
        match self {
            DBUniError::ConnectionFailed(_) => "connection_failed",
            DBUniError::ConnectionLost(_) => "connection_lost",
            DBUniError::AuthenticationFailed(_) => "authentication_failed",
            DBUniError::Timeout(_) => "timeout",
            DBUniError::SyntaxError { .. } => "syntax_error",
            DBUniError::NotFound { .. } => "not_found",
            DBUniError::UniqueViolation(_) => "unique_violation",
            DBUniError::ForeignKeyViolation(_) => "foreign_key_violation",
            DBUniError::NotNullViolation(_) => "not_null_violation",
            DBUniError::CheckViolation(_) => "check_violation",
            DBUniError::Deadlock(_) => "deadlock",
            DBUniError::TransactionAborted(_) => "transaction_aborted",
            DBUniError::TypeMismatch { .. } => "type_mismatch",
            DBUniError::OutOfRange(_) => "out_of_range",
            DBUniError::NoRowsReturned => "no_rows_returned",
            DBUniError::TooManyRowsReturned { .. } => "too_many_rows_returned",
            DBUniError::Unsupported(_) => "unsupported",
            DBUniError::NotImplemented(_) => "not_implemented",
            DBUniError::ParameterError(_) => "parameter_error",
            DBUniError::Driver { .. } => "driver",
            DBUniError::Serialization(_) => "serialization",
            DBUniError::Io(_) => "io",
            DBUniError::Cancelled => "cancelled",
            DBUniError::Other(_) => "other",
        }
    }
    /// Returns `true` if the error is likely transient and the operation may be retried.
    pub const fn is_retryable(&self) -> bool {
        matches!(
            self,
            DBUniError::ConnectionLost(_)
                | DBUniError::Timeout(_)
                | DBUniError::Deadlock(_)
                | DBUniError::TransactionAborted(_)
        )
    }
    /// Returns `true` if the error indicates a constraint violation.
    pub const fn is_constraint_violation(&self) -> bool {
        matches!(
            self,
            DBUniError::UniqueViolation(_)
                | DBUniError::ForeignKeyViolation(_)
                | DBUniError::NotNullViolation(_)
                | DBUniError::CheckViolation(_)
        )
    }
}
impl fmt::Display for DBUniError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DBUniError::ConnectionFailed(msg) => write!(f, "connection failed: {msg}"),
            DBUniError::ConnectionLost(msg) => write!(f, "connection lost: {msg}"),
            DBUniError::AuthenticationFailed(msg) => write!(f, "authentication failed: {msg}"),
            DBUniError::Timeout(msg) => write!(f, "timeout: {msg}"),
            DBUniError::SyntaxError { statement, message } => {
                write!(f, "syntax error in `{statement}`: {message}")
            }
            DBUniError::NotFound { object, name } => {
                write!(f, "{object} not found: {name}")
            }
            DBUniError::UniqueViolation(msg) => write!(f, "unique violation: {msg}"),
            DBUniError::ForeignKeyViolation(msg) => write!(f, "foreign key violation: {msg}"),
            DBUniError::NotNullViolation(msg) => write!(f, "not null violation: {msg}"),
            DBUniError::CheckViolation(msg) => write!(f, "check violation: {msg}"),
            DBUniError::Deadlock(msg) => write!(f, "deadlock: {msg}"),
            DBUniError::TransactionAborted(msg) => write!(f, "transaction aborted: {msg}"),
            DBUniError::TypeMismatch { from, to } => {
                write!(f, "type mismatch: cannot convert {from} to {to}")
            }
            DBUniError::OutOfRange(msg) => write!(f, "value out of range: {msg}"),
            DBUniError::NoRowsReturned => write!(f, "no rows returned"),
            DBUniError::TooManyRowsReturned { actual } => {
                write!(f, "too many rows returned: {actual}")
            }
            DBUniError::Unsupported(msg) => write!(f, "unsupported: {msg}"),
            DBUniError::NotImplemented(msg) => write!(f, "not implemented: {msg}"),
            DBUniError::ParameterError(msg) => write!(f, "parameter error: {msg}"),
            DBUniError::Driver {
                database,
                code,
                message,
            } => match code {
                Some(code) => write!(f, "[{database}] driver error ({code}): {message}"),
                None => write!(f, "[{database}] driver error: {message}"),
            },
            DBUniError::Serialization(msg) => write!(f, "serialization error: {msg}"),
            DBUniError::Io(msg) => write!(f, "io error: {msg}"),
            DBUniError::Cancelled => write!(f, "operation cancelled"),
            DBUniError::Other(msg) => write!(f, "{msg}"),
        }
    }
}
impl std::error::Error for DBUniError {}
/// A convenient alias for results produced by dbuni operations.
pub type DBUniResultT<T> = Result<T, DBUniError>;
