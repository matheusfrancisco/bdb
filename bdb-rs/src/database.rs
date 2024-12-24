use crate::entry::Entry;
use crate::page::{Page, PageHeader};
use itertools::Itertools;
use std::fs;
use std::path::Path;

pub struct Database {
    buffer: Vec<u8>,
}

impl Database {
    pub fn open(filename: impl AsRef<Path>) -> std::io::Result<Self> {
        let buffer = fs::read(filename)?;
        Ok(Self { buffer })
    }
    pub fn close(self) {}

    pub fn stat_print(&self) {
        for page in self.pages() {
            println!("{:#?}", page.header);
        }
        println!("Key-Value Data:");
        for (key, value) in self.walk() {
            println!("   Key: {key}");
            println!("   Value: {value}");
        }
    }

    pub fn pages(&self) -> impl Iterator<Item = Page<'_>> {
        self.raw_pages().map(Page::new)
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        let mut current_page = self.pages().find_map(|p| {
            if let PageHeader::Metadata { root, .. } = p.header {
                Some(root as usize)
            } else {
                None
            }
        })?;
        loop {
            let page = self.page(current_page)?;
            if page.is_internal() {
                let mut prev: Option<usize> = None;
                for entry in page.entries() {
                    if let Entry::Internal { pgno, data, .. } = entry {
                        if key >= data {
                            prev = Some(pgno as usize);
                        } else {
                            break;
                        }
                    } else {
                        unreachable!();
                    }
                }
                current_page = prev?;
            } else {
                let mut offset = None;
                for (idx, entry) in page.entries().enumerate().step_by(2) {
                    let Entry::KeyData { data: key_data } = entry else {
                        unreachable!();
                    };
                    if key == key_data {
                        offset = Some(idx);
                    }
                    if key <= key_data {
                        break;
                    }
                }
                if let Entry::KeyData {
                    data: value_data, ..
                } = page.get_entry(offset? + 1)?
                {
                    return Some(value_data);
                };
                break;
            }
        }
        None
    }

    fn page(&self, index: usize) -> Option<Page<'_>> {
        let start_offset = index * 4096;
        let end_offset = start_offset + 4096;
        if end_offset <= self.buffer.len() {
            Some(Page::new(&self.buffer[start_offset..end_offset]))
        } else {
            None
        }
    }

    // iterate over the raw pages of the database
    pub fn raw_pages(&self) -> impl Iterator<Item = &[u8]> {
        self.buffer.chunks(4096)
    }

    pub fn walk(&self) -> impl Iterator<Item = (Entry<'_>, Entry<'_>)> {
        Walk::new(self)
    }
}

struct Walk<'a> {
    page: usize,
    entry: usize,
    pages: Vec<Page<'a>>,
}

impl<'a> Walk<'a> {
    fn new(database: &'a Database) -> Self {
        let pages = database.pages().collect();
        Self {
            page: 0,
            entry: 0,
            pages,
        }
    }
    fn current_page(&self) -> &Page<'a> {
        &self.pages[self.page]
    }

    fn move_to_leaf(&mut self) {
        loop {
            let page = self.current_page();
            match page.header {
                // is_leaf()
                PageHeader::BTree { level: 1, .. } => {
                    break;
                }
                PageHeader::BTree { .. } => {
                    let Entry::Internal { pgno, .. } = page.entries().next().unwrap() else {
                        unreachable!()
                    };
                    self.entry = 0;
                    self.page = pgno as usize;
                }
                PageHeader::Metadata { root, .. } => {
                    self.entry = 0;
                    self.page = root as usize;
                }
            }
        }
    }
}

impl<'a> Iterator for Walk<'a> {
    type Item = (Entry<'a>, Entry<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        //(k,v)
        //1. try to read k => Some(..)
        //    Try to read v => Some(..) => return Some(k,v)
        //2. try to read k=> None
        //     are we at the last page?
        //     otherwise move to the next page and try to read
        //
        // if we have consumed all of the entries in the last leaf page
        // we must return None.
        self.move_to_leaf();
        let mut page = self.page;
        let mut entry = self.entry;
        let mut result;
        loop {
            let current_page = &self.pages[page];
            result = current_page
                .get_entry(entry)
                .zip(current_page.get_entry(entry + 1));
            // NOTE: we ignore when there are uneven numbers of entries,
            // assuming we should try the next page
            if result.is_none() {
                if let Some(next) = current_page.next_page_number() {
                    page = next as usize;
                    entry = 0;
                    continue;
                }
            }
            entry += 2;
            break;
        }
        self.page = page;
        self.entry = entry;
        result
    }
}
