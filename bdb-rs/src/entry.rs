use std::{fmt::Display, str::from_utf8};

#[derive(Debug, Clone)]
pub enum Entry<'a> {
    KeyData {
        data: &'a [u8],
    },
    Internal {
        ty: u8,
        pgno: u32,
        nrecs: u32,
        data: &'a [u8],
    },
}

impl<'a> Display for Entry<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KeyData { data, .. } => {
                if let Ok(s) = from_utf8(data) {
                    write!(f, "{s}")
                } else {
                    write!(f, "({} binary data)", data.len())
                }
            }
            Self::Internal {
                ty,
                pgno,
                nrecs,
                data,
            } => {
                let key = if let Ok(s) = from_utf8(data) {
                    s
                } else {
                    &format!("({} binary data)", data.len())
                };
                write!(
                    f,
                    "Type: {ty}, Page: {pgno}, Records: {nrecs}, Minimum key: {key}"
                )
            }
        }
    }
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
            ty,
            pgno,
            nrecs,
            data,
        }
    }
}
