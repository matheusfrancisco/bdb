use std::env;
mod database;
mod entry;
mod page;

use database::Database;

//https://transactional.blog/building-berkeleydb/page-format

fn main() {
    let filename = env::args().nth(1).unwrap();
    println!("Reading file: {filename}");

    let db = Database::open(filename);
    for page in db.pages() {
        println!("Page: {:?}", page.header);
        for (idx, entry) in page.entries().enumerate() {
            if page.is_leaf() {
                let key_or_value = if idx % 2 == 0 { "Key" } else { "Value" };
                println!("{key_or_value} {:?}", entry);
            } else {
                println!("  Internal {:?}", entry);
            }
        }
    }
}
