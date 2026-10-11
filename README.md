<h1 align="center">dbuni</h1>
<h4 align="center">A unified database abstraction layer.</h4>

<p align="center">
  <a href="https://github.com/0xhappyboy/dbuni/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-Apache2.0-d1d1f6.svg?style=flat&labelColor=1C2C2E&color=BEC5C9&logo=googledocs&label=license&logoColor=BEC5C9" alt="License"></a>
  <a href="https://crates.io/crates/dbuni"><img src="https://img.shields.io/badge/crates-dbuni-20B2AA.svg?style=flat&labelColor=0F1F2D&color=FFD700&logo=rust&logoColor=FFD700"></a>
  <a href="https://crates.io/crates/dbuni"><img src="https://img.shields.io/crates/d/dbuni?style=flat&labelColor=0F1F2D&color=20B2AA&logo=rust&logoColor=white&label=downloads" alt="Crates.io Downloads"></a>
  <a href="https://github.com/0xhappyboy/dbuni"><img src="https://img.shields.io/badge/github-dbuni-181717.svg?style=flat&labelColor=0F1F2D&color=20B2AA&logo=github&logoColor=white" alt="GitHub"></a>
</p>

<p align="center">
<a href="./README_zh-CN.md">简体中文</a> | <a href="./README.md">English</a>
</p>

## Overview

**dbuni is a unified database connection layer for Rust.**

It hides the differences between database clients behind one type, so
your code connects, queries, and handles results the same way across
multiple databases and multiple data models.

You write:

- one connection call: `DBUniClient::connect(database, options)`
- one query call: `query_raw` / `query_rows` / `query_one_as` / `query_all_as` / `execute`
- one result type: `DBUniResult`
- one error type: `DBUniError`
- one connection state: `DBUniConnectionState`

The underlying driver library is never exposed. Adding a new database
means adding a driver, not changing the client.

## Demo

```rust
use dbuni::{DBUniClient, DBUniConnectOptions, DBUniDatabase, DBUniParam};

// mysql
let sql_opts = DBUniConnectOptions::new("127.0.0.1", 3306, "user", "secret", "mydb")
    .with_option("max_connections", "5");
let mut sql_client = DBUniClient::connect(DBUniDatabase::MySql, &sql_opts)?;
let result = sql_client.query_raw(
    "SELECT id, name FROM users WHERE id = ?",
    &[DBUniParam::Int(1)],
)?;
println!("sql  kind    = {:?}", result.kind);    // Rows
println!("sql  payload = {:?}", result.payload);
sql_client.close()?;

// redis
let kv_opts = DBUniConnectOptions::new("127.0.0.1", 6379, "", "secret", "0");
let mut kv_client = DBUniClient::connect(DBUniDatabase::Redis, &kv_opts)?;
kv_client.query_raw("SET", &["user:1".into(), "alice".into()])?;
let result = kv_client.query_raw("GET", &["user:1".into()])?;
println!("kv   kind    = {:?}", result.kind);    // Scalar
println!("kv   payload = {:?}", result.payload);
kv_client.close()?;

// mongodb
let doc_opts = DBUniConnectOptions::new("127.0.0.1", 27017, "root", "secret", "mydb");
let mut doc_client = DBUniClient::connect(DBUniDatabase::MongoDb, &doc_opts)?;
let result = doc_client.query_raw(
    r#"{"collection": "users", "filter": {"_id": 1}}"#,
    &[],
)?;
println!("doc  kind    = {:?}", result.kind);    // Document
println!("doc  payload = {:?}", result.payload);
doc_client.close()?;

// sqlite
let embed_opts = DBUniConnectOptions::new_embedded("data/app.db")
    .with_option("max_connections", "1");
let mut embed_client = DBUniClient::connect(DBUniDatabase::SQLite, &embed_opts)?;
let result = embed_client.query_raw(
    "SELECT id, name FROM users WHERE id = ?",
    &[DBUniParam::Int(1)],
)?;
println!("emb  kind    = {:?}", result.kind);    // Rows
println!("emb  payload = {:?}", result.payload);
embed_client.close()?;
```
