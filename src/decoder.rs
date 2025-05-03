use std::collections::HashMap;
use std::io::{self, Read};
use flate2::read::ZlibDecoder;
use heatshrink::{decode as heatshrink_decode, Config};
use std::str;
use thiserror::Error;

#[derive(Debug)]
pub enum BlockType {
    FileMetadata,
    GCode,
    SlicerMetadata,
    PrinterMetadata,
    PrintMetadata,
    Thumbnail,
}

impl BlockType {
    fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(Self::FileMetadata),
            1 => Some(Self::GCode),
            2 => Some(Self::SlicerMetadata),
            3 => Some(Self::PrinterMetadata),
            4 => Some(Self::PrintMetadata),
            5 => Some(Self::Thumbnail),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Checksum {
    None,
    CRC32,
}

impl Checksum {
    fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::CRC32),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Compression {
    None,
    Deflate,
    Heatshrink11_4,
    Heatshrink12_4,
}

impl Compression {
    fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Deflate),
            2 => Some(Self::Heatshrink11_4),
            3 => Some(Self::Heatshrink12_4),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ImageFormat {
    PNG,
    JPG,
    QOI,
}

impl ImageFormat {
    fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(Self::PNG),
            1 => Some(Self::JPG),
            2 => Some(Self::QOI),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct FileMetadata {
    pub data: String,
}

#[derive(Debug)]
pub struct GCode {
    pub data: String,
}

#[derive(Debug)]
pub struct SlicerMetadata {
    pub data: HashMap<String, String>,
}

#[derive(Debug)]
pub struct PrinterMetadata {
    pub data: HashMap<String, String>,
}

#[derive(Debug)]
pub struct PrintMetadata {
    pub data: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Thumbnail {
    pub format: ImageFormat,
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("Invalid file format")]
    InvalidFile,
    #[error("Unsupported GCode version")]
    UnsupportedVersion,
    #[error("Invalid block type")]
    InvalidBlockType,
    #[error("Invalid compression type")]
    InvalidCompression,
    #[error("Invalid size")]
    InvalidSize,
    #[error("I/O error")]
    Io(#[from] io::Error),
}

fn read_u16<R: Read>(reader: &mut R) -> Result<u16, DecodeError> {
    let mut buffer = [0u8; 2];
    reader.read_exact(&mut buffer)?;
    Ok(u16::from_le_bytes(buffer))
}

fn read_u32<R: Read>(reader: &mut R) -> Result<u32, DecodeError> {
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

fn parse_metadata(data: &str) -> HashMap<String, String> {
    data.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let mut value = parts[1].trim().replace("\\n", "\n");
                if value.starts_with('"') && value.ends_with('"') {
                    value = value[1..value.len() - 1].to_string();
                }
                Some((key, value))
            } else {
                None
            }
        })
        .collect()
}

pub fn decode<R: Read>(mut reader: R) -> Result<Vec<Box<dyn std::fmt::Debug>>, DecodeError> {
    let mut blocks: Vec<Box<dyn std::fmt::Debug>> = Vec::new();

    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if &magic != b"GCDE" {
        return Err(DecodeError::InvalidFile);
    }

    let version = read_u32(&mut reader)?;
    if version != 1 {
        return Err(DecodeError::UnsupportedVersion);
    }

    let checksum_type = Checksum::from_u16(read_u16(&mut reader)?).ok_or(DecodeError::InvalidBlockType)?;

    loop {
        let block_type = match BlockType::from_u16(read_u16(&mut reader)?) {
            Some(b) => b,
            None => break,
        };

        let compression = Compression::from_u16(read_u16(&mut reader)?).ok_or(DecodeError::InvalidCompression)?;
        let uncompressed_size = read_u32(&mut reader)? as usize;
        let size = if compression != Compression::None {
            read_u32(&mut reader)? as usize
        } else {
            uncompressed_size
        };

        if uncompressed_size == 0 {
            return Err(DecodeError::InvalidSize);
        }

        let (format, width, height) = if let BlockType::Thumbnail = block_type {
            (
                ImageFormat::from_u16(read_u16(&mut reader)?).ok_or(DecodeError::InvalidBlockType)?,
                read_u16(&mut reader)?,
                read_u16(&mut reader)?,
            )
        } else {
            (ImageFormat::PNG, 0, 0) // Defaults for non-thumbnail blocks
        };

        let mut data = vec![0; size];
        reader.read_exact(&mut data)?;

        if checksum_type == Checksum::CRC32 {
            let _checksum = read_u32(&mut reader)?; // TODO: Implement checksum validation
        }

        let data = match compression {
            Compression::Deflate => {
                let mut decompressed = Vec::new();
                ZlibDecoder::new(&data[..]).read_to_end(&mut decompressed)?;
                decompressed
            }
            Compression::Heatshrink11_4 | Compression::Heatshrink12_4 => {
                let mut output = vec![0; uncompressed_size]; // Allocate buffer
                match heatshrink_decode(&data, &mut output, &Config::default()) {
                    Ok(decoded_data) => decoded_data.to_vec(),
                    Err(_) => Vec::new(),
                }
            }
            Compression::None => data,
        };

        let decoded_block: Box<dyn std::fmt::Debug> = match block_type {
            BlockType::FileMetadata => Box::new(FileMetadata { data: String::from_utf8_lossy(&data).into_owned() }),
            BlockType::GCode => Box::new(GCode { data: String::from_utf8_lossy(&data).into_owned() }),
            BlockType::SlicerMetadata | BlockType::PrinterMetadata | BlockType::PrintMetadata => {
                let metadata = parse_metadata(&String::from_utf8_lossy(&data));
                Box::new(SlicerMetadata { data: metadata })
            }
            BlockType::Thumbnail => Box::new(Thumbnail { format, width, height, data }),
        };

        blocks.push(decoded_block);
    }

    Ok(blocks)
}
