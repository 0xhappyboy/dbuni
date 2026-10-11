use dbuni::{
    DBUniClient, DBUniConnectOptions, DBUniConnectionState, DBUniDatabase, DBUniError,
    DBUniFromRow, DBUniParam, DBUniRawRow, DBUniResultT, DBUniValue,
};
struct User {
    id: i64,
    name: String,
}
impl DBUniFromRow for User {
    fn from_row(row: &DBUniRawRow) -> DBUniResultT<Self> {
        let id_value = row.get_by_name("id").or_else(|| row.get_by_name("_id"));
        let id = match id_value {
            Some(DBUniValue::Int(v)) => *v,
            Some(DBUniValue::UInt(v)) => *v as i64,
            other => {
                return Err(DBUniError::TypeMismatch {
                    from: format!("{other:?}"),
                    to: "i64".to_string(),
                });
            }
        };
        let name = match row.get_by_name("name") {
            Some(DBUniValue::Text(v)) => v.clone(),
            other => {
                return Err(DBUniError::TypeMismatch {
                    from: format!("{other:?}"),
                    to: "String".to_string(),
                });
            }
        };
        Ok(User { id, name })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_multi_database_chaos() {
        // MySQL
        let mysql_options =
            DBUniConnectOptions::new("127.0.0.1", 3306, "root", "root123456", "testdb")
                .with_option("max_connections", "5");
        let mut mysql = DBUniClient::connect(DBUniDatabase::MySql, &mysql_options)
            .expect("failed to connect to MySQL");
        // Redis
        let redis_options = DBUniConnectOptions::new("127.0.0.1", 6379, "", "redis123456", "0");
        let mut redis = DBUniClient::connect(DBUniDatabase::Redis, &redis_options)
            .expect("failed to connect to Redis");
        // SQLite
        let sqlite_options = DBUniConnectOptions::new_embedded(
            r"C:\Users\admin\AppData\Roaming\HippoX\data\sessions.db",
        )
        .with_option("max_connections", "1");
        let mut sqlite = DBUniClient::connect(DBUniDatabase::SQLite, &sqlite_options)
            .expect("failed to connect to SQLite");
        // PostgreSQL
        let postgres_options =
            DBUniConnectOptions::new("127.0.0.1", 5432, "postgres", "postgres123456", "testdb")
                .with_option("max_connections", "5");
        let mut postgres = DBUniClient::connect(DBUniDatabase::PostgreSQL, &postgres_options)
            .expect("failed to connect to PostgreSQL");
        // MongoDB
        let mongodb_options =
            DBUniConnectOptions::new("127.0.0.1", 27017, "root", "root123456", "testdb");
        let mut mongodb = DBUniClient::connect(DBUniDatabase::MongoDb, &mongodb_options)
            .expect("failed to connect to MongoDB");
        println!("mysql    engine = {}", mysql.database());
        println!("redis    engine = {}", redis.database());
        println!("sqlite   engine = {}", sqlite.database());
        println!("postgres engine = {}", postgres.database());
        println!("mongodb  engine = {}", mongodb.database());
        assert_eq!(mysql.database(), DBUniDatabase::MySql);
        assert_eq!(redis.database(), DBUniDatabase::Redis);
        assert_eq!(sqlite.database(), DBUniDatabase::SQLite);
        assert_eq!(postgres.database(), DBUniDatabase::PostgreSQL);
        assert_eq!(mongodb.database(), DBUniDatabase::MongoDb);
        assert_eq!(mysql.state(), DBUniConnectionState::Connected);
        assert_eq!(redis.state(), DBUniConnectionState::Connected);
        assert_eq!(sqlite.state(), DBUniConnectionState::Connected);
        assert_eq!(postgres.state(), DBUniConnectionState::Connected);
        assert_eq!(mongodb.state(), DBUniConnectionState::Connected);
        println!("sqlite path     = {}", sqlite.options().path);
        mysql
            .execute("CREATE DATABASE IF NOT EXISTS dbuni_chaos", &[])
            .expect("mysql create database failed");
        println!("mysql: database `dbuni_chaos` ensured");
        let databases = mysql
            .query_rows("SHOW DATABASES", &[])
            .expect("mysql list databases failed");
        println!("mysql: {} databases", databases.len());
        for row in &databases {
            if let Some(value) = row.get_by_name("Database") {
                println!("  db = {:?}", value);
            }
        }
        mysql
            .execute(
                "CREATE TABLE IF NOT EXISTS dbuni_chaos.t_user (
                    id INT PRIMARY KEY,
                    name VARCHAR(64)
                )",
                &[],
            )
            .expect("mysql create table failed");
        println!("mysql: table `dbuni_chaos.t_user` ensured");
        mysql
            .execute(
                "REPLACE INTO dbuni_chaos.t_user (id, name) VALUES (?, ?)",
                &[DBUniParam::Int(1), DBUniParam::Text("alice".to_string())],
            )
            .expect("mysql insert 1 failed");
        mysql
            .execute(
                "REPLACE INTO dbuni_chaos.t_user (id, name) VALUES (?, ?)",
                &[DBUniParam::Int(2), DBUniParam::Text("bob".to_string())],
            )
            .expect("mysql insert 2 failed");
        let affected = mysql
            .execute(
                "UPDATE dbuni_chaos.t_user SET name = ? WHERE id = ?",
                &[DBUniParam::Text("alice2".to_string()), DBUniParam::Int(1)],
            )
            .expect("mysql update failed");
        println!("mysql: update affected = {affected}");
        let rows = mysql
            .query_rows("SELECT id, name FROM dbuni_chaos.t_user ORDER BY id", &[])
            .expect("mysql select failed");
        println!("mysql: {} rows in `t_user`", rows.len());
        for row in &rows {
            println!("  row = {:?}", row);
        }
        assert_eq!(rows.len(), 2);
        let tables = mysql
            .query_rows("SHOW TABLES FROM dbuni_chaos", &[])
            .expect("mysql list tables failed");
        println!("mysql: {} tables in `dbuni_chaos`", tables.len());
        for row in &tables {
            println!("  table = {:?}", row.get(0));
        }
        let mysql_result = mysql
            .query_raw("SELECT id, name FROM dbuni_chaos.t_user", &[])
            .expect("mysql query_raw failed");
        println!("mysql kind    = {:?}", mysql_result.kind);
        println!("mysql payload = {:?}", mysql_result.payload);
        assert!(mysql_result.is_rows());
        let one: Option<User> = mysql
            .query_one_as(
                "SELECT id, name FROM dbuni_chaos.t_user WHERE id = ?",
                &[DBUniParam::Int(1)],
            )
            .expect("mysql query_one_as failed");
        match &one {
            Some(u) => println!("mysql typed one  = id={}, name={}", u.id, u.name),
            None => println!("mysql typed one  = none"),
        }
        assert!(one.is_some());
        assert_eq!(one.as_ref().unwrap().id, 1);
        let all: Vec<User> = mysql
            .query_all_as("SELECT id, name FROM dbuni_chaos.t_user ORDER BY id", &[])
            .expect("mysql query_all_as failed");
        println!("mysql typed all  = {} users", all.len());
        for u in &all {
            println!("  user = id={}, name={}", u.id, u.name);
        }
        assert_eq!(all.len(), 2);
        let db_count = redis
            .query_raw("CONFIG", &["GET".into(), "databases".into()])
            .expect("redis CONFIG GET databases failed");
        println!("redis: CONFIG GET databases -> {:?}", db_count.payload);
        redis
            .query_raw("SET", &["dbuni:a".into(), "alpha".into()])
            .expect("redis SET a failed");
        redis
            .query_raw("SET", &["dbuni:b".into(), "beta".into()])
            .expect("redis SET b failed");
        redis
            .query_raw(
                "HSET",
                &[
                    "dbuni:hash".into(),
                    "f1".into(),
                    "v1".into(),
                    "f2".into(),
                    "v2".into(),
                ],
            )
            .expect("redis HSET failed");
        redis
            .query_raw("SET", &["dbuni:a".into(), "alpha2".into()])
            .expect("redis SET a (update) failed");
        let keys = redis
            .query_raw("KEYS", &["*".into()])
            .expect("redis KEYS failed");
        println!("redis: KEYS * -> {:?}", keys.payload);
        if let dbuni::DBUniPayload::Array(items) = &keys.payload {
            for item in items {
                if let dbuni::DBUniValue::Text(key) = item {
                    if !key.starts_with("dbuni:") {
                        continue;
                    }
                    let type_result = redis
                        .query_raw("TYPE", &[key.as_str().into()])
                        .expect("redis TYPE failed");
                    let type_name = match &type_result.payload {
                        dbuni::DBUniPayload::Scalar(dbuni::DBUniValue::Text(t)) => t.clone(),
                        _ => "unknown".to_string(),
                    };
                    println!("  type  {} -> {}", key, type_name);
                    match type_name.as_str() {
                        "string" => {
                            let value = redis
                                .query_raw("GET", &[key.as_str().into()])
                                .expect("redis GET failed");
                            println!("  value {} -> {:?}", key, value.payload);
                        }
                        "hash" => {
                            let value = redis
                                .query_raw("HGETALL", &[key.as_str().into()])
                                .expect("redis HGETALL failed");
                            println!("  value {} -> {:?}", key, value.payload);
                        }
                        other => {
                            println!("  value {} -> (unsupported type: {})", key, other);
                        }
                    }
                }
            }
        }
        sqlite
            .execute(
                "CREATE TABLE IF NOT EXISTS dbuni_sqlite_test (
                    id INTEGER PRIMARY KEY,
                    name TEXT
                )",
                &[],
            )
            .expect("sqlite create table failed");
        sqlite
            .execute(
                "REPLACE INTO dbuni_sqlite_test (id, name) VALUES (?, ?)",
                &[DBUniParam::Int(1), DBUniParam::Text("alice".to_string())],
            )
            .expect("sqlite insert 1 failed");
        sqlite
            .execute(
                "REPLACE INTO dbuni_sqlite_test (id, name) VALUES (?, ?)",
                &[DBUniParam::Int(2), DBUniParam::Text("bob".to_string())],
            )
            .expect("sqlite insert 2 failed");
        let affected = sqlite
            .execute(
                "UPDATE dbuni_sqlite_test SET name = ? WHERE id = ?",
                &[DBUniParam::Text("bob2".to_string()), DBUniParam::Int(2)],
            )
            .expect("sqlite update failed");
        println!("sqlite: update affected = {affected}");
        let rows = sqlite
            .query_rows("SELECT id, name FROM dbuni_sqlite_test ORDER BY id", &[])
            .expect("sqlite select failed");
        println!("sqlite: {} rows in `dbuni_sqlite_test`", rows.len());
        for row in &rows {
            println!("  row = {:?}", row);
        }
        assert_eq!(rows.len(), 2);
        let tables = sqlite
            .query_rows("SELECT name FROM sqlite_master WHERE type = 'table'", &[])
            .expect("sqlite list tables failed");
        println!("sqlite: {} tables", tables.len());
        for row in &tables {
            println!("  table = {:?}", row.get(0));
        }
        let sqlite_result = sqlite
            .query_raw("SELECT id, name FROM dbuni_sqlite_test", &[])
            .expect("sqlite query_raw failed");
        println!("sqlite kind    = {:?}", sqlite_result.kind);
        println!("sqlite payload = {:?}", sqlite_result.payload);
        assert!(sqlite_result.is_rows());
        let sqlite_one: Option<User> = sqlite
            .query_one_as(
                "SELECT id, name FROM dbuni_sqlite_test WHERE id = ?",
                &[DBUniParam::Int(1)],
            )
            .expect("sqlite query_one_as failed");
        match &sqlite_one {
            Some(u) => println!("sqlite typed one = id={}, name={}", u.id, u.name),
            None => println!("sqlite typed one = none"),
        }
        assert!(sqlite_one.is_some());
        let sqlite_all: Vec<User> = sqlite
            .query_all_as("SELECT id, name FROM dbuni_sqlite_test ORDER BY id", &[])
            .expect("sqlite query_all_as failed");
        println!("sqlite typed all = {} users", sqlite_all.len());
        for u in &sqlite_all {
            println!("  user = id={}, name={}", u.id, u.name);
        }
        assert_eq!(sqlite_all.len(), 2);
        postgres
            .execute(
                "CREATE TABLE IF NOT EXISTS dbuni_pg_test (
                    id INTEGER PRIMARY KEY,
                    name TEXT
                )",
                &[],
            )
            .expect("postgres create table failed");
        println!("postgres: table `dbuni_pg_test` ensured");
        postgres
            .execute(
                "INSERT INTO dbuni_pg_test (id, name) VALUES ($1, $2)
                 ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name",
                &[DBUniParam::Int(1), DBUniParam::Text("alice".to_string())],
            )
            .expect("postgres insert 1 failed");
        postgres
            .execute(
                "INSERT INTO dbuni_pg_test (id, name) VALUES ($1, $2)
                 ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name",
                &[DBUniParam::Int(2), DBUniParam::Text("bob".to_string())],
            )
            .expect("postgres insert 2 failed");
        let affected = postgres
            .execute(
                "UPDATE dbuni_pg_test SET name = $1 WHERE id = $2",
                &[DBUniParam::Text("alice2".to_string()), DBUniParam::Int(1)],
            )
            .expect("postgres update failed");
        println!("postgres: update affected = {affected}");
        let rows = postgres
            .query_rows("SELECT id, name FROM dbuni_pg_test ORDER BY id", &[])
            .expect("postgres select failed");
        println!("postgres: {} rows in `dbuni_pg_test`", rows.len());
        for row in &rows {
            println!("  row = {:?}", row);
        }
        assert_eq!(rows.len(), 2);
        let databases = postgres
            .query_rows("SELECT datname FROM pg_database ORDER BY datname", &[])
            .expect("postgres list databases failed");
        println!("postgres: {} databases", databases.len());
        for row in &databases {
            println!("  db = {:?}", row.get(0));
        }
        let tables = postgres
            .query_rows(
                "SELECT table_name FROM information_schema.tables
                 WHERE table_schema = 'public' ORDER BY table_name",
                &[],
            )
            .expect("postgres list tables failed");
        println!("postgres: {} tables", tables.len());
        for row in &tables {
            println!("  table = {:?}", row.get(0));
        }
        let pg_result = postgres
            .query_raw("SELECT id, name FROM dbuni_pg_test", &[])
            .expect("postgres query_raw failed");
        println!("postgres kind    = {:?}", pg_result.kind);
        println!("postgres payload = {:?}", pg_result.payload);
        assert!(pg_result.is_rows());
        let pg_one: Option<User> = postgres
            .query_one_as(
                "SELECT id, name FROM dbuni_pg_test WHERE id = $1",
                &[DBUniParam::Int(1)],
            )
            .expect("postgres query_one_as failed");
        match &pg_one {
            Some(u) => println!("postgres typed one = id={}, name={}", u.id, u.name),
            None => println!("postgres typed one = none"),
        }
        assert!(pg_one.is_some());
        let pg_all: Vec<User> = postgres
            .query_all_as("SELECT id, name FROM dbuni_pg_test ORDER BY id", &[])
            .expect("postgres query_all_as failed");
        println!("postgres typed all = {} users", pg_all.len());
        for u in &pg_all {
            println!("  user = id={}, name={}", u.id, u.name);
        }
        assert_eq!(pg_all.len(), 2);
        mongodb
            .execute(
                r#"{"op": "delete", "collection": "dbuni_mongo_test", "filter": {}}"#,
                &[],
            )
            .expect("mongodb drop failed");
        mongodb
            .execute(
                r#"{"op": "insert", "collection": "dbuni_mongo_test"}"#,
                &[DBUniParam::Json(
                    r#"{"_id": 1, "name": "alice", "age": 30}"#.to_string(),
                )],
            )
            .expect("mongodb insert 1 failed");
        mongodb
            .execute(
                r#"{"op": "insert", "collection": "dbuni_mongo_test"}"#,
                &[DBUniParam::Json(
                    r#"{"_id": 2, "name": "bob", "age": 25}"#.to_string(),
                )],
            )
            .expect("mongodb insert 2 failed");
        mongodb
            .execute(
                r#"{"op": "update", "collection": "dbuni_mongo_test", "filter": {"_id": 1}}"#,
                &[DBUniParam::Json(
                    r#"{"$set": {"name": "alice2"}}"#.to_string(),
                )],
            )
            .expect("mongodb update failed");
        let mongo_result = mongodb
            .query_raw(r#"{"collection": "dbuni_mongo_test", "filter": {}}"#, &[])
            .expect("mongodb query_raw failed");
        println!("mongodb kind    = {:?}", mongo_result.kind);
        println!("mongodb payload = {:?}", mongo_result.payload);
        assert!(matches!(
            mongo_result.kind,
            dbuni::DBUniResponseKind::Document
        ));
        let mongo_rows = mongodb
            .query_rows(r#"{"collection": "dbuni_mongo_test", "filter": {}}"#, &[])
            .expect("mongodb query_rows failed");
        println!("mongodb: {} documents", mongo_rows.len());
        for row in &mongo_rows {
            println!("  row = {:?}", row);
        }
        let mongo_one: Option<User> = mongodb
            .query_one_as(
                r#"{"collection": "dbuni_mongo_test", "filter": {"_id": 1}}"#,
                &[],
            )
            .expect("mongodb query_one_as failed");
        match &mongo_one {
            Some(u) => println!("mongodb typed one = id={}, name={}", u.id, u.name),
            None => println!("mongodb typed one = none"),
        }
        if let Some(u) = &mongo_one {
            assert_eq!(u.name, "alice2");
        }
        let mongo_all: Vec<User> = mongodb
            .query_all_as(r#"{"collection": "dbuni_mongo_test", "filter": {}}"#, &[])
            .expect("mongodb query_all_as failed");
        println!("mongodb typed all = {} users", mongo_all.len());
        for u in &mongo_all {
            println!("  user = id={}, name={}", u.id, u.name);
        }
        let mysql_name = mysql
            .query_rows(
                "SELECT name FROM dbuni_chaos.t_user WHERE id = ?",
                &[DBUniParam::Int(1)],
            )
            .expect("mysql cross read failed");
        println!("cross mysql    id=1 name = {:?}", mysql_name.get(0));
        let sqlite_name = sqlite
            .query_rows(
                "SELECT name FROM dbuni_sqlite_test WHERE id = ?",
                &[DBUniParam::Int(1)],
            )
            .expect("sqlite cross read failed");
        println!("cross sqlite   id=1 name = {:?}", sqlite_name.get(0));
        let postgres_name = postgres
            .query_rows(
                "SELECT name FROM dbuni_pg_test WHERE id = $1",
                &[DBUniParam::Int(1)],
            )
            .expect("postgres cross read failed");
        println!("cross postgres id=1 name = {:?}", postgres_name.get(0));
        let redis_name = redis
            .query_raw("GET", &["dbuni:a".into()])
            .expect("redis cross read failed");
        println!("cross redis    dbuni:a   = {:?}", redis_name.payload);
        let mongo_name = mongodb
            .query_raw(
                r#"{"collection": "dbuni_mongo_test", "filter": {"_id": 1}}"#,
                &[],
            )
            .expect("mongodb cross read failed");
        println!("cross mongodb  _id=1     = {:?}", mongo_name.payload);
        // Clean up everything the test created.
        mysql
            .execute("DROP DATABASE IF EXISTS dbuni_chaos", &[])
            .expect("mysql drop database failed");
        println!("mysql: database `dbuni_chaos` dropped");
        redis
            .execute("DEL", &["dbuni:a".into()])
            .expect("redis DEL a failed");
        redis
            .execute("DEL", &["dbuni:b".into()])
            .expect("redis DEL b failed");
        redis
            .execute("DEL", &["dbuni:hash".into()])
            .expect("redis DEL hash failed");
        println!("redis: seeded keys deleted");
        sqlite
            .execute("DROP TABLE IF EXISTS dbuni_sqlite_test", &[])
            .expect("sqlite drop table failed");
        println!("sqlite: table `dbuni_sqlite_test` dropped");
        postgres
            .execute("DROP TABLE IF EXISTS dbuni_pg_test", &[])
            .expect("postgres drop table failed");
        println!("postgres: table `dbuni_pg_test` dropped");
        mongodb
            .execute(
                r#"{"op": "delete", "collection": "dbuni_mongo_test", "filter": {}}"#,
                &[],
            )
            .expect("mongodb drop failed");
        println!("mongodb: documents in `dbuni_mongo_test` deleted");
        mysql.close().expect("mysql close failed");
        redis.close().expect("redis close failed");
        sqlite.close().expect("sqlite close failed");
        postgres.close().expect("postgres close failed");
        mongodb.close().expect("mongodb close failed");
        assert_eq!(mysql.state(), DBUniConnectionState::Closed);
        assert_eq!(redis.state(), DBUniConnectionState::Closed);
        assert_eq!(sqlite.state(), DBUniConnectionState::Closed);
        assert_eq!(postgres.state(), DBUniConnectionState::Closed);
        assert_eq!(mongodb.state(), DBUniConnectionState::Closed);
        println!("all five connections closed");
        let _ = DBUniParam::Null;
    }
}
