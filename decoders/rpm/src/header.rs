// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::io::{Read, Seek, SeekFrom};

use upac_types::error::DecodeError;

use super::rpm::LEAD_SIZE;

const LEAD_MAGIC: [u8; 4] = [0xED, 0xAB, 0xEE, 0xDB];
const SECTION_MAGIC: [u8; 3] = [0x8E, 0xAD, 0xE8];

struct SectionHeader {
    tag_count: u32,
    data_size: u32,
}

impl SectionHeader {
    fn read<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let mut buffer = [0u8; 16];
        reader
            .read_exact(&mut buffer)
            .map_err(|_| DecodeError::MalformedMetadata)?;

        if buffer[0..3] != SECTION_MAGIC {
            return Err(DecodeError::MalformedMetadata);
        }

        Ok(SectionHeader {
            tag_count: u32::from_be_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]),
            data_size: u32::from_be_bytes([buffer[12], buffer[13], buffer[14], buffer[15]]),
        })
    }

    fn skip_body<R: Read + Seek>(&self, reader: &mut R) -> Result<(), DecodeError> {
        let total_size = u64::from(self.tag_count) * 16 + u64::from(self.data_size);
        reader.seek(SeekFrom::Current(total_size as i64))?;

        let remainder = total_size % 8;
        if remainder != 0 {
            reader.seek(SeekFrom::Current((8 - remainder) as i64))?;
        }

        Ok(())
    }
}

struct TagEntry {
    tag: u32,
    offset: u32,
    count: u32,
}

impl TagEntry {
    fn from_bytes(chunk: &[u8; 16]) -> Self {
        TagEntry {
            tag: u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
            offset: u32::from_be_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]),
            count: u32::from_be_bytes([chunk[12], chunk[13], chunk[14], chunk[15]]),
        }
    }
}

struct RawData(Vec<u8>);

impl RawData {
    fn read<R: Read>(reader: &mut R, size: usize) -> Result<Self, DecodeError> {
        let mut bytes = vec![0u8; size];
        reader
            .read_exact(&mut bytes)
            .map_err(|_| DecodeError::MalformedMetadata)?;

        Ok(RawData(bytes))
    }

    fn string_at(&self, offset: usize) -> Result<String, DecodeError> {
        let slice = self.0.get(offset..).ok_or(DecodeError::MalformedMetadata)?;
        let end = slice
            .iter()
            .position(|&byte| byte == 0)
            .ok_or(DecodeError::MalformedMetadata)?;

        String::from_utf8(slice[..end].to_vec()).map_err(|_| DecodeError::InvalidUtf8)
    }

    fn i32_at(&self, offset: usize) -> Result<i32, DecodeError> {
        let bytes = self.0.get(offset..offset + 4).ok_or(DecodeError::MalformedMetadata)?;

        Ok(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
}

pub struct Header {
    entries: Vec<TagEntry>,
    data: RawData,
}

impl Header {
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self, DecodeError> {
        Self::skip_lead(reader)?;
        SectionHeader::read(reader)?.skip_body(reader)?;
        Self::read_main(reader)
    }

    pub fn string(&self, tag: u32) -> Result<Option<String>, DecodeError> {
        let Some(entry) = self.find(tag) else { return Ok(None) };

        self.data.string_at(entry.offset as usize).map(Some)
    }

    pub fn string_array(&self, tag: u32) -> Result<Vec<String>, DecodeError> {
        let Some(entry) = self.find(tag) else {
            return Ok(Vec::new());
        };

        let mut values = Vec::with_capacity(entry.count as usize);
        let mut cursor = entry.offset as usize;
        for _ in 0..entry.count {
            let value = self.data.string_at(cursor)?;
            cursor += value.len() + 1;
            values.push(value);
        }

        Ok(values)
    }

    pub fn int32(&self, tag: u32) -> Result<Option<i32>, DecodeError> {
        let Some(entry) = self.find(tag) else { return Ok(None) };

        self.data.i32_at(entry.offset as usize).map(Some)
    }

    pub fn int32_array(&self, tag: u32) -> Result<Vec<i32>, DecodeError> {
        let Some(entry) = self.find(tag) else {
            return Ok(Vec::new());
        };

        (0..entry.count as usize)
            .map(|index| self.data.i32_at(entry.offset as usize + index * 4))
            .collect()
    }

    pub fn contains(&self, tag: u32) -> bool {
        self.find(tag).is_some()
    }

    fn find(&self, tag: u32) -> Option<&TagEntry> {
        self.entries.iter().find(|entry| entry.tag == tag)
    }

    fn skip_lead<R: Read + Seek>(reader: &mut R) -> Result<(), DecodeError> {
        let mut magic = [0u8; 4];
        reader
            .read_exact(&mut magic)
            .map_err(|_| DecodeError::UnsupportedFormat)?;

        if magic != LEAD_MAGIC {
            return Err(DecodeError::UnsupportedFormat);
        }

        reader.seek(SeekFrom::Start(u64::from(LEAD_SIZE)))?;

        Ok(())
    }

    fn read_main<R: Read>(reader: &mut R) -> Result<Self, DecodeError> {
        let header = SectionHeader::read(reader)?;

        let mut index_bytes = vec![0u8; header.tag_count as usize * 16];
        reader
            .read_exact(&mut index_bytes)
            .map_err(|_| DecodeError::MalformedMetadata)?;

        let data = RawData::read(reader, header.data_size as usize)?;

        let entries = index_bytes
            .as_chunks::<16>()
            .0
            .iter()
            .map(TagEntry::from_bytes)
            .collect();

        Ok(Header { entries, data })
    }
}
