// https://datatracker.ietf.org/doc/html/rfc8878#name-compression-algorithm

use std::error::Error;

pub type Res<T> = Result<T, Box<dyn Error>>;

pub const MAGIC_NUMBER: u32 = 0xFD2FB528;

struct Frame {
    frame_header: FrameHeader,
    data_blocks: Vec<Block>,
    content_checksum: Option<u32>,
}

struct FrameHeader {
    window_size: u64,
    dictionary_id: u32,
    frame_content_size: u64,
}

struct Block {
    block_type: BlockType,
    block_content: Vec<u8>,
}

enum BlockType {
    Raw,
    Rle,
    Compressed,
    Reserved,
}

pub struct Zstd {
    frames: Vec<Frame>,
}

impl Zstd {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        let magic_number = u32::from_ne_bytes(bytes[0..4].try_into()?);

        if magic_number != MAGIC_NUMBER {
            return Err("Invalid magic number".into());
        }

        Ok(Self { frames: vec![] })
    }

    pub fn encode(bytes: Vec<u8>) -> Vec<u8> {
        vec![]
    }
    pub fn decode(&self) -> Vec<u8> {
        vec![]
    }
}
