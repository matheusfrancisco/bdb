use std::fmt::{self, Debug, Formatter};

use crate::entry::Entry;

pub struct Page<'a> {
    pub header: PageHeader<'a>,
    // this is borrowed from the buffer in Database
    data: &'a [u8],
}

impl<'a> Page<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        if data.len() < 4096 {
            panic!("Invalid page length {}", data.len());
        }

        let header = PageHeader::new(&data);
        Self { header, data }
    }

    pub fn get_entry(&self, index: usize) -> Option<Entry<'a>> {
        let PageHeader::BTree { entries, level, .. } = self.header else {
            return None;
        };

        if index >= entries as usize {
            return None;
        }

        let prev_offset = if index == 0 {
            4096
        } else {
            self.get_offset(index - 1)
        };
        let offset = self.get_offset(index);
        let buffer = &self.data[offset..prev_offset];
        let entry = if level == 1 {
            Entry::new_keydata(buffer)
        } else {
            Entry::new_internal(buffer)
        };
        Some(entry)
    }

    fn get_offset(&self, index: usize) -> usize {
        u16::from_le_bytes(
            self.data[(26 + 2 * index)..(28 + 2 * index)]
                .try_into()
                .unwrap(),
        ) as usize
    }

    pub fn entries(&'a self) -> impl Iterator<Item = Entry<'a>> {
        EntryInterator::new(self)
    }

    pub fn is_metadata(&self) -> bool {
        self.header.is_metadata()
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self.header, PageHeader::BTree { level: 1, .. })
    }

    pub fn is_internal(&self) -> bool {
        !(self.is_metadata() || self.is_leaf())
    }

    pub fn next_page_number(&self) -> Option<u32> {
        if let PageHeader::BTree { next_pgno, .. } = self.header {
            if next_pgno != 0 {
                return Some(next_pgno);
            }
        }
        None
    }
}

struct EntryInterator<'a> {
    current: usize,
    end: usize,
    page: &'a Page<'a>,
}

impl<'a> EntryInterator<'a> {
    fn new(page: &'a Page<'a>) -> Self {
        Self {
            current: 0,
            end: 4096,
            page,
        }
    }
}

impl<'a> Iterator for EntryInterator<'a> {
    type Item = Entry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let PageHeader::BTree { entries, level, .. } = self.page.header else {
            return None;
        };

        if entries as usize <= self.current {
            return None;
        };

        let offset_bytes = self.page.data[(26 + 2 * self.current)..(28 + 2 * self.current)]
            .try_into()
            .unwrap();
        let offset = u16::from_le_bytes(offset_bytes);

        let buffer = &self.page.data[offset as usize..self.end];
        let entry = if level == 1 {
            Entry::new_keydata(buffer)
        } else {
            Entry::new_internal(buffer)
        };
        self.current += 1;
        //next entry start at the end of the current entry
        self.end = offset as usize;
        Some(entry)
    }
}

impl<'a> Debug for Page<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Page")
            .field("header", &self.header)
            .finish()
    }
}

#[allow(dead_code)]
#[derive(Debug, derive_more::IsVariant)]
pub enum PageHeader<'a> {
    Metadata {
        lsn: u64,
        pgno: u32,
        magic: u32,
        version: u32,
        pagesize: u32,
        ec: u8,
        ty: u8,
        mf: u8,
        //empty byte
        free: u32,
        last_pgno: u32,
        nparts: u32,
        key_count: u32,
        record_count: u32,
        flags: u32,
        uid: &'a [u8],
        //empty word
        minkey: u32,
        re_len: u32,
        re_pad: u32,
        root: u32,
        crypto_magic: u32,
        iv: u128,
        chksum: &'a [u8],
    },
    BTree {
        lsn: u64,
        pgno: u32,
        prev_pgno: u32,
        next_pgno: u32,
        entries: u16,
        hf_offset: u16,
        level: u8,
        ty: u8,
        // offsets: Vec<u16>,
    },
}

impl<'a> PageHeader<'a> {
    // Metadata page marker
    const MAGIC_BYTES: &'static [u8] = &u32::to_le_bytes(0x00053162);

    // BTree page types
    const TYPE_META: u8 = 9;
    const TYPE_INERNAL: u8 = 3;
    const TYPE_LEAF: u8 = 5;

    fn new(data: &'a [u8]) -> Self {
        if &data[12..16] == Self::MAGIC_BYTES {
            Self::new_metadata(data)
        } else {
            Self::new_btree(data)
        }
    }

    fn new_metadata(data: &'a [u8]) -> Self {
        let magic = 0x00053162;
        let lsn = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let pgno = u32::from_le_bytes(data[8..12].try_into().unwrap());
        let version = u32::from_le_bytes(data[16..20].try_into().unwrap());
        let pagesize = u32::from_le_bytes(data[20..24].try_into().unwrap());
        let ec = data[24];
        let ty = data[25];
        let mf = data[26];
        let free = u32::from_le_bytes(data[28..32].try_into().unwrap());
        let last_pgno = u32::from_le_bytes(data[32..36].try_into().unwrap());
        let nparts = u32::from_le_bytes(data[36..40].try_into().unwrap());
        let key_count = u32::from_le_bytes(data[40..44].try_into().unwrap());
        let record_count = u32::from_le_bytes(data[44..48].try_into().unwrap());
        let flags = u32::from_le_bytes(data[48..52].try_into().unwrap());
        let uid = &data[52..72];
        let minkey = u32::from_le_bytes(data[76..80].try_into().unwrap());
        let re_len = u32::from_le_bytes(data[80..84].try_into().unwrap());
        let re_pad = u32::from_le_bytes(data[84..88].try_into().unwrap());
        let root = u32::from_le_bytes(data[88..92].try_into().unwrap());
        let crypto_magic = u32::from_le_bytes(data[460..464].try_into().unwrap());
        let iv = u128::from_le_bytes(data[476..492].try_into().unwrap());
        let chksum = &data[492..512];

        Self::Metadata {
            lsn,
            pgno,
            magic,
            version,
            pagesize,
            ec,
            ty,
            mf,
            free,
            last_pgno,
            nparts,
            key_count,
            record_count,
            flags,
            uid,
            minkey,
            re_len,
            re_pad,
            root,
            crypto_magic,
            iv,
            chksum,
        }
    }

    fn new_btree(data: &'a [u8]) -> Self {
        let lsn = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let pgno = u32::from_le_bytes(data[8..12].try_into().unwrap());
        let prev_pgno = u32::from_le_bytes(data[12..16].try_into().unwrap());
        let next_pgno = u32::from_le_bytes(data[16..20].try_into().unwrap());
        let entries = u16::from_le_bytes(data[20..22].try_into().unwrap());
        let hf_offset = u16::from_le_bytes(data[22..24].try_into().unwrap());
        let level = data[24];
        let ty = data[25];

        if !(ty == Self::TYPE_META || ty == Self::TYPE_LEAF || ty == Self::TYPE_INERNAL) {
            panic!("Invalid BTree page type {ty}");
        }

        Self::BTree {
            lsn,
            pgno,
            prev_pgno,
            next_pgno,
            entries,
            hf_offset,
            level,
            ty,
        }
    }
}
