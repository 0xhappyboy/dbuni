pub mod types;
pub use types::*;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn category_mapping_is_total() {
        // Every variant must map to a category without panicking.
        println!("\n=== category_mapping_is_total ===");
        for db in DatabaseType::all() {
            let cat = db.category();
            println!(
                "{:<20} key={:<20} category={:<15} ({})",
                db.as_str(),
                db.as_key(),
                cat.as_str(),
                cat.as_key(),
            );
        }
        for db in DatabaseType::all() {
            let _ = db.category();
        }
    }
    #[test]
    fn key_roundtrip() {
        println!("\n=== key_roundtrip ===");
        for db in DatabaseType::all() {
            let key = db.as_key();
            let parsed = DatabaseType::from_key(key);
            println!(
                "{:<20} key={:<20} roundtrip={:<20} ok={}",
                db.as_str(),
                key,
                parsed.as_str(),
                parsed == *db,
            );
            assert_eq!(
                DatabaseType::from_key(key),
                *db,
                "roundtrip failed for {key}"
            );
        }
    }
    #[test]
    fn unknown_key_falls_back_to_other() {
        println!("\n=== unknown_key_falls_back_to_other ===");
        let inputs = ["not-a-db", "", "MYSQL", "postgres", "unknown-engine"];
        for input in inputs {
            let parsed = DatabaseType::from_key(input);
            println!(
                "input={:<20} parsed={:<20} key={}",
                format!("{:?}", input),
                parsed.as_str(),
                parsed.as_key(),
            );
            assert_eq!(parsed, DatabaseType::Other);
        }
    }
    #[test]
    fn known_keys_match_frontend() {
        println!("\n=== known_keys_match_frontend ===");
        let cases: &[(&str, DatabaseType)] = &[
            ("mysql", DatabaseType::MySql),
            ("postgresql", DatabaseType::PostgreSQL),
            ("mongodb", DatabaseType::MongoDb),
            ("redis", DatabaseType::Redis),
            ("duckdb-emb", DatabaseType::DuckDbEmbedded),
        ];
        for (key, expected) in cases {
            let parsed = DatabaseType::from_key(key);
            println!(
                "key={:<20} expected={:<20} parsed={:<20} ok={}",
                key,
                expected.as_str(),
                parsed.as_str(),
                parsed == *expected,
            );
            assert_eq!(parsed, *expected);
        }
    }
    #[test]
    fn dump_all_databases_en_zh() {
        println!("\n=== dump_all_databases_en_zh ===");
        println!(
            "{:<20} {:<20} {:<20} {:<20}",
            "key", "English", "Chinese", "Category"
        );
        println!("{}", "-".repeat(85));
        for db in DatabaseType::all() {
            println!(
                "{:<20} {:<20} {:<20} {:<20}",
                db.as_key(),
                db.as_str(),
                db.as_str_zh(),
                db.category().as_key(),
            );
        }
        println!("total = {}", DatabaseType::all().len());
    }
    #[test]
    fn dump_all_categories() {
        println!("\n=== dump_all_categories ===");
        let categories = [
            DatabaseCategory::Sql,
            DatabaseCategory::NoSql,
            DatabaseCategory::Cloud,
            DatabaseCategory::Embedded,
            DatabaseCategory::TimeSeries,
            DatabaseCategory::Graph,
            DatabaseCategory::Search,
            DatabaseCategory::KeyValue,
            DatabaseCategory::WideColumn,
            DatabaseCategory::ObjectStorage,
            DatabaseCategory::MessageQueue,
            DatabaseCategory::Other,
        ];
        println!("{:<20} {:<20}", "key", "label");
        println!("{}", "-".repeat(40));
        for cat in categories {
            println!("{:<20} {:<20}", cat.as_key(), cat.as_str());
        }
        println!("total = {}", categories.len());
    }
}
