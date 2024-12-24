use std::env;

mod database;
mod entry;
mod page;

use database::Database;

//https://transactional.blog/building-berkeleydb/page-format

fn main() -> std::io::Result<()> {
    let filename = env::args().nth(1).expect("Please pass a database filename");
    let db = Database::open(filename)?;
    db.stat_print();
    println!("==================================");
    test_get(&db, "bbbbbbbbbbbbbbbbb");
    test_get(&db, "kjshdfkhjdsfhdsj");
    test_get(&db, "sssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssss");
    db.close();
    Ok(())
}

fn test_get(db: &Database, key: &str) {
    let bytes = key.as_bytes();
    match db.get(bytes) {
        Some(value) => {
            let value_str = std::str::from_utf8(value).unwrap();
            println!("key: {key}, data: {value_str}");
        }
        None => {
            println!("key not found");
        }
    }
}
