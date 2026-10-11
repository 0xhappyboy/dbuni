use dbuni::{DBUniClient, DBUniConnectOptions, DBUniConnectionState, DBUniDatabase, DBUniParam};
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mysql_and_redis_together() {
        // MySQL
        let mysql_options =
            DBUniConnectOptions::new("127.0.0.1", 3306, "root", "root123456", "testdb")
                .with_option("max_connections", "5");
        let mut mysql = DBUniClient::connect(DBUniDatabase::MySql, &mysql_options)
            .expect("failed to connect to MySQL");
        println!("mysql database = {}", mysql.database());
        println!("mysql state    = {}", mysql.state());
        assert_eq!(mysql.database(), DBUniDatabase::MySql);
        assert_eq!(mysql.state(), DBUniConnectionState::Connected);
        let mysql_result = mysql
            .query_raw("SELECT 1", &[])
            .expect("mysql query_raw failed");
        println!("mysql kind     = {:?}", mysql_result.kind);
        println!("mysql payload  = {:?}", mysql_result.payload);
        // Redis
        let redis_options = DBUniConnectOptions::new("127.0.0.1", 6379, "", "redis123456", "0");
        let mut redis = DBUniClient::connect(DBUniDatabase::Redis, &redis_options)
            .expect("failed to connect to Redis");
        println!("redis database = {}", redis.database());
        println!("redis state    = {}", redis.state());
        assert_eq!(redis.database(), DBUniDatabase::Redis);
        assert_eq!(redis.state(), DBUniConnectionState::Connected);
        // SET then GET, to exercise both the Ok and Scalar payload shapes.
        redis
            .query_raw("SET", &["dbuni:test".into(), "hello".into()])
            .expect("redis SET failed");
        let redis_result = redis
            .query_raw("GET", &["dbuni:test".into()])
            .expect("redis GET failed");
        println!("redis kind     = {:?}", redis_result.kind);
        println!("redis payload  = {:?}", redis_result.payload);
        redis
            .execute("DEL", &["dbuni:test".into()])
            .expect("redis DEL failed");
        // Close both
        mysql.close().expect("mysql close failed");
        redis.close().expect("redis close failed");
        assert_eq!(mysql.state(), DBUniConnectionState::Closed);
        assert_eq!(redis.state(), DBUniConnectionState::Closed);
        // Reference the parameter type so the import stays used.
        let _ = DBUniParam::Null;
    }
    #[test]
    fn test_mysql_metadata_and_redis_keyspace() {
        // MySQL: DDL + metadata
        let mysql_options =
            DBUniConnectOptions::new("127.0.0.1", 3306, "root", "root123456", "testdb")
                .with_option("max_connections", "5");
        let mut mysql = DBUniClient::connect(DBUniDatabase::MySql, &mysql_options)
            .expect("failed to connect to MySQL");
        mysql
            .execute("CREATE DATABASE IF NOT EXISTS dbuni_meta_test", &[])
            .expect("create database failed");
        println!("mysql: database `dbuni_meta_test` ensured");
        let databases = mysql
            .query_rows("SHOW DATABASES", &[])
            .expect("list databases failed");
        println!("mysql: {} databases", databases.len());
        for row in &databases {
            // Column name from `SHOW DATABASES` is `Database`.
            if let Some(value) = row.get_by_name("Database") {
                println!("  db = {:?}", value);
            }
        }
        mysql
            .execute(
                "CREATE TABLE IF NOT EXISTS dbuni_meta_test.t_user (
                    id INT PRIMARY KEY,
                    name VARCHAR(64)
                )",
                &[],
            )
            .expect("create table failed");
        println!("mysql: table `dbuni_meta_test.t_user` ensured");
        mysql
            .execute(
                "REPLACE INTO dbuni_meta_test.t_user (id, name) VALUES (?, ?)",
                &[DBUniParam::Int(1), DBUniParam::Text("alice".to_string())],
            )
            .expect("insert failed");
        let tables = mysql
            .query_rows("SHOW TABLES FROM dbuni_meta_test", &[])
            .expect("list tables failed");
        println!("mysql: {} tables in `dbuni_meta_test`", tables.len());
        for row in &tables {
            let value = row.get(0).cloned();
            println!("  table = {:?}", value);
        }
        let rows = mysql
            .query_rows("SELECT id, name FROM dbuni_meta_test.t_user", &[])
            .expect("select failed");
        println!("mysql: {} rows in `t_user`", rows.len());
        for row in &rows {
            println!("  row = {:?}", row);
        }
        mysql
            .execute("DROP DATABASE IF EXISTS dbuni_meta_test", &[])
            .expect("drop database failed");
        println!("mysql: database `dbuni_meta_test` dropped");
        let redis_options = DBUniConnectOptions::new("127.0.0.1", 6379, "", "redis123456", "0");
        let mut redis = DBUniClient::connect(DBUniDatabase::Redis, &redis_options)
            .expect("failed to connect to Redis");
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
        let keys = redis
            .query_raw("KEYS", &["*".into()])
            .expect("redis KEYS failed");
        println!("redis: KEYS * -> {:?}", keys.payload);
        if let dbuni::DBUniPayload::Array(items) = &keys.payload {
            for item in items {
                if let dbuni::DBUniValue::Text(key) = item {
                    // Skip keys we did not seed, so the test stays
                    // independent of whatever else lives in the db.
                    if !key.starts_with("dbuni:") {
                        continue;
                    }
                    // Ask Redis for the key's type.
                    let type_result = redis
                        .query_raw("TYPE", &[key.as_str().into()])
                        .expect("redis TYPE failed");
                    let type_name = match &type_result.payload {
                        dbuni::DBUniPayload::Scalar(dbuni::DBUniValue::Text(t)) => t.clone(),
                        _ => "unknown".to_string(),
                    };
                    println!("  type  {} -> {}", key, type_name);
                    // Pick the right reader for the type.
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
        redis
            .execute("DEL", &["dbuni:a".into()])
            .expect("redis DEL a failed");
        redis
            .execute("DEL", &["dbuni:b".into()])
            .expect("redis DEL b failed");
        redis
            .execute("DEL", &["dbuni:hash".into()])
            .expect("redis DEL hash failed");
        mysql.close().expect("mysql close failed");
        redis.close().expect("redis close failed");
        assert_eq!(mysql.state(), DBUniConnectionState::Closed);
        assert_eq!(redis.state(), DBUniConnectionState::Closed);
    }
}
