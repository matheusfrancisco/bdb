use crate::page::Page;
use std::fs;
use std::path::Path;

pub struct Database {
    buffer: Vec<u8>,
}

impl Database {
    pub fn open(filename: impl AsRef<Path>) -> Self {
        let buffer = fs::read(filename).unwrap();
        Self { buffer }
    }

    pub fn pages(&self) -> impl Iterator<Item = Page<'_>> {
        self.raw_pages().map(Page::new)
    }

    // iterate over the raw pages of the database
    pub fn raw_pages(&self) -> impl Iterator<Item = &[u8]> {
        self.buffer.chunks(4096)
    }
}
