#[derive(Debug, Clone)]
pub enum Entry<'a> {
    KeyData {
        length: u16,
        data: &'a [u8],
    },
    Internal {
        length: u16,
        ty: u8,
        pgno: u32,
        nrecs: u32,
        data: &'a [u8],
    },
}

impl<'a> Entry<'a> {
    pub fn new_keydata(buffer: &'a [u8]) -> Self {
        if buffer.len() < 3 {
            panic!("Invalid entry");
        }
        if buffer[2] != 1 {
            panic!("Invalid entry type");
        }

        let length = u16::from_le_bytes(buffer[0..2].try_into().unwrap());
        Self::KeyData {
            length,
            data: &buffer[3..length as usize],
        }
    }

    pub fn new_internal(buffer: &'a [u8]) -> Self {
        if buffer.len() < 12 {
            panic!("Invalid entry length {}", buffer.len());
        }
        let length = u16::from_le_bytes(buffer[0..2].try_into().unwrap());
        let ty = buffer[2];
        let pgno = u32::from_le_bytes(buffer[4..8].try_into().unwrap());
        let nrecs = u32::from_le_bytes(buffer[8..12].try_into().unwrap());
        let data = &buffer[12..(length + 12) as usize];
        Self::Internal {
            length,
            ty,
            pgno,
            nrecs,
            data,
        }
    }
}
