use std::env;
mod database;
mod page;
mod entry;

use database::Database;

//https://transactional.blog/building-berkeleydb/page-format

fn main() {
    let filename = env::args().nth(1).unwrap();
    println!("Reading file: {filename}");

    let db = Database::open(filename);
    for page in db.pages() {
        println!("Page: {:?}", page.header);
        for entry in page.entries() {
            println!("Entry: {:?}", entry);
        }
    }
}
