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
    offsets: Vec<u16>,
}

#[derive(Debug, Clone)]
enum Entry<'a> {
    KeyData {
        length: u16,
        data: &'a [u8],
    },
    Internal {
        length: u16,
        pgno: u32,
        nrecs: u32,
        data: &'a [u8],
    },
    Overflow {
        pgno: u32,
        tlen: u32,
    },
}

impl<'a> Entry<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        if buffer.len() < 3 {
            panic!("Invalid entry");
        }
        match buffer[2] {
            1 => {
                let length = u16::from_le_bytes(buffer[0..2].try_into().unwrap());
                Self::KeyData {
                    length,
                    data: &buffer[3..length as usize],
                }
            }
            _ => unimplemented!(),
        }
    }
}

impl BTreePageHeader {
    const TYPE_META: u8 = 9;
    const TYPE_INERNAL: u8 = 3;
    const TYPE_LEAF: u8 = 5;

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

        if !(ty == Self::TYPE_META || ty == Self::TYPE_LEAF || ty == Self::TYPE_INERNAL) {
            panic!("Invalid page type {ty}");
        }

        let offsets = (0..entries as usize)
            .map(|e| {
                u16::from_le_bytes(page[(26 + 2 * e)..(28 + 2 * e)].try_into().unwrap()) - hf_offset
            })
            .collect();

        Self {
            lsn,
            pgno,
            prev_pgno,
            next_pgno,
            entries,
            hf_offset,
            level,
            ty,
            offsets,
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
    for page in contents[4096..].chunks(4096) {
        let header = BTreePageHeader::new(page);
        if header.ty == BTreePageHeader::TYPE_LEAF {
            for pair in header.offsets.chunks(2) {
                let key_offset = (pair[0] + header.hf_offset) as usize;
                let value_offset = (pair[1] + header.hf_offset) as usize;

                let key = Entry::new(&page[key_offset..]);
                let value = Entry::new(&page[value_offset..]);
                println!("Key: {:?}, --> Value: {:?}", key, value);
            }
        }
    }
}
