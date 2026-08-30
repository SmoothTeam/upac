// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

pub trait RedbCodable: Sized {
    fn redb_encode(&self, buf: &mut Vec<u8>);
    fn redb_decode(data: &[u8], offset: &mut usize) -> Self;
}

impl RedbCodable for String {
    fn redb_encode(&self, buf: &mut Vec<u8>) {
        write_len_prefixed(buf, self.as_bytes());
    }

    fn redb_decode(data: &[u8], offset: &mut usize) -> Self {
        read_str(data, offset)
    }
}

impl RedbCodable for u32 {
    fn redb_encode(&self, buf: &mut Vec<u8>) {
        write_u32(buf, *self);
    }

    fn redb_decode(data: &[u8], offset: &mut usize) -> Self {
        read_u32(data, offset)
    }
}

impl RedbCodable for u64 {
    fn redb_encode(&self, buf: &mut Vec<u8>) {
        write_u64(buf, *self);
    }

    fn redb_decode(data: &[u8], offset: &mut usize) -> Self {
        read_u64(data, offset)
    }
}

impl RedbCodable for bool {
    fn redb_encode(&self, buf: &mut Vec<u8>) {
        write_bool(buf, *self);
    }

    fn redb_decode(data: &[u8], offset: &mut usize) -> Self {
        read_bool(data, offset)
    }
}

impl<T: RedbCodable> RedbCodable for Option<T> {
    fn redb_encode(&self, buf: &mut Vec<u8>) {
        match self {
            Some(value) => {
                buf.push(1);
                value.redb_encode(buf);
            }
            None => buf.push(0),
        }
    }

    fn redb_decode(data: &[u8], offset: &mut usize) -> Self {
        let flag = data[*offset];
        *offset += 1;

        if flag == 1 {
            Some(T::redb_decode(data, offset))
        } else {
            None
        }
    }
}

impl<T: RedbCodable> RedbCodable for Vec<T> {
    fn redb_encode(&self, buf: &mut Vec<u8>) {
        write_u32(buf, self.len() as u32);

        for element in self {
            element.redb_encode(buf);
        }
    }

    fn redb_decode(data: &[u8], offset: &mut usize) -> Self {
        let len = read_u32(data, offset);

        (0..len).map(|_| T::redb_decode(data, offset)).collect()
    }
}

pub(crate) fn write_bool(buf: &mut Vec<u8>, value: bool) {
    buf.push(u8::from(value));
}

pub(crate) fn write_u32(buf: &mut Vec<u8>, value: u32) {
    buf.extend_from_slice(&value.to_le_bytes());
}

pub(crate) fn write_u64(buf: &mut Vec<u8>, value: u64) {
    buf.extend_from_slice(&value.to_le_bytes());
}

pub fn write_len_prefixed(buf: &mut Vec<u8>, bytes: &[u8]) {
    write_u32(buf, bytes.len() as u32);
    buf.extend_from_slice(bytes);
}

pub fn write_opt_str(buf: &mut Vec<u8>, value: Option<&str>) {
    match value {
        Some(text) => {
            buf.push(1);
            write_len_prefixed(buf, text.as_bytes());
        }
        None => buf.push(0),
    }
}

pub(crate) fn read_bool(data: &[u8], offset: &mut usize) -> bool {
    let value = data[*offset] != 0;
    *offset += 1;

    value
}

pub(crate) fn read_u32(data: &[u8], offset: &mut usize) -> u32 {
    let value = u32::from_le_bytes(data[*offset..*offset + 4].try_into().unwrap());
    *offset += 4;

    value
}

pub(crate) fn read_u64(data: &[u8], offset: &mut usize) -> u64 {
    let value = u64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
    *offset += 8;

    value
}

pub(crate) fn read_len_prefixed<'a>(data: &'a [u8], offset: &mut usize) -> &'a [u8] {
    let len = read_u32(data, offset) as usize;

    let bytes = &data[*offset..*offset + len];
    *offset += len;

    bytes
}

pub(crate) fn read_str(data: &[u8], offset: &mut usize) -> String {
    String::from_utf8_lossy(read_len_prefixed(data, offset)).into_owned()
}
