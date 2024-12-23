use std::env;
use std::fs;

//https://transactional.blog/building-berkeleydb/page-format

#[derive(Debug, Clone, Default)]
struct Metadata<'a> {
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
}

impl<'a> Metadata<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        if buffer.len() < 512 {
            panic!("Invalid metadata page");
        }

        let bytes = buffer[12..16].try_into().unwrap();

        println!("{:?}", bytes);
        let magic = u32::from_le_bytes(bytes);

        if magic != 0x00053162 {
            panic!("Invalid magic number {magic:x}");
        }

        let lsn = u64::from_le_bytes(buffer[0..8].try_into().unwrap());
        let pgno = u32::from_le_bytes(buffer[8..12].try_into().unwrap());
        let version = u32::from_le_bytes(buffer[16..20].try_into().unwrap());
        let pagesize = u32::from_le_bytes(buffer[20..24].try_into().unwrap());
        let ec = buffer[24];
        let ty = buffer[25];
        let mf = buffer[26];
        let free = u32::from_le_bytes(buffer[28..32].try_into().unwrap());
        let last_pgno = u32::from_le_bytes(buffer[32..36].try_into().unwrap());
        let nparts = u32::from_le_bytes(buffer[36..40].try_into().unwrap());
        let key_count = u32::from_le_bytes(buffer[40..44].try_into().unwrap());
        let record_count = u32::from_le_bytes(buffer[44..48].try_into().unwrap());
        let flags = u32::from_le_bytes(buffer[48..52].try_into().unwrap());
        let uid = &buffer[52..72];
        let minkey = u32::from_le_bytes(buffer[76..80].try_into().unwrap());
        let re_len = u32::from_le_bytes(buffer[80..84].try_into().unwrap());
        let re_pad = u32::from_le_bytes(buffer[84..88].try_into().unwrap());
        let root = u32::from_le_bytes(buffer[88..92].try_into().unwrap());
        let crypto_magic = u32::from_le_bytes(buffer[460..464].try_into().unwrap());
        let iv = u128::from_le_bytes(buffer[476..492].try_into().unwrap());
        let chksum = &buffer[492..512];

        Self {
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
}

#[derive(Debug, Clone)]
struct BTreePageHeader {
    lsn: u64,
    pgno: u32,
    prev_pgno: u32,
    next_pgno: u32,
    entries: u16,
    hf_offset: u16,
    level: u8,
    ty: u8,
}

impl BTreePageHeader {
    fn new(page: &[u8]) -> Self {
        if page.len() < 4096 {
            panic!("Invalid page size {}", page.len());
        }
        let lsn = u64::from_le_bytes(page[0..8].try_into().unwrap());
        let pgno = u32::from_le_bytes(page[8..12].try_into().unwrap());
        let prev_pgno = u32::from_le_bytes(page[12..16].try_into().unwrap());
        let next_pgno = u32::from_le_bytes(page[16..20].try_into().unwrap());
        let entries = u16::from_le_bytes(page[20..22].try_into().unwrap());
        let hf_offset = u16::from_le_bytes(page[22..24].try_into().unwrap());
        let level = page[24];
        let ty = page[25];

        if !(ty == 9 || ty == 3 || ty == 5) {
            panic!("Invalid page type {ty}");
        }

        Self {
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

fn main() {
    let filename = env::args().nth(1).unwrap();
    println!("Reading file: {filename}");
    let contents = fs::read(filename).unwrap();
    let metadata_contents = &contents[0..512];
    let metadata = Metadata::new(metadata_contents);
    println!("{metadata:#?}");
    for chunk in contents[4096..].chunks(4096) {
        let header = BTreePageHeader::new(chunk);
        println!("{header:#?}");
    }
}
