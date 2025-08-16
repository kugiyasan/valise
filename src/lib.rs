// https://datatracker.ietf.org/doc/html/rfc8878#name-compression-algorithm

use std::error::Error;

pub type Res<T> = Result<T, Box<dyn Error>>;

pub const MAGIC_NUMBER: u32 = 0xFD2FB528;

#[derive(Debug)]
struct Frame {
    frame_header: FrameHeader,
    data_blocks: Vec<Block>,
    content_checksum: Option<u32>,
}

impl Frame {
    fn from_bytes(mut bytes: &[u8]) -> Res<Self> {
        let magic_number = u32::from_le_bytes(bytes[0..4].try_into()?);
        bytes = &bytes[4..];

        if magic_number != MAGIC_NUMBER {
            return Err("Invalid magic number".into());
        }

        let frame_header = FrameHeader::from_bytes(bytes)?;
        bytes = &bytes[frame_header.len..];

        let mut data_blocks = vec![];
        loop {
            let block = Block::from_bytes(bytes)?;
            bytes = &bytes[block.len()..];
            let is_last_block = block.block_header.is_last_block();

            data_blocks.push(block);

            if is_last_block {
                break;
            }
        }

        let content_checksum = if frame_header.frame_header_descriptor.content_checksum_flag() {
            Some(u32::from_le_bytes(bytes[..4].try_into()?))
        } else {
            None
        };

        let frame = Frame {
            frame_header,
            data_blocks,
            content_checksum,
        };

        Ok(frame)
    }

    fn len(&self) -> usize {
        let data_blocks_len = self
            .data_blocks
            .iter()
            .map(|block| block.len())
            .sum::<usize>();

        let content_checksum_len = if self.content_checksum.is_some() {
            4
        } else {
            0
        };

        4 + self.frame_header.len + data_blocks_len + content_checksum_len
    }

    fn decode(self) -> Vec<u8> {
        self.data_blocks
            .into_iter()
            .map(|block| block.decode())
            .flatten()
            .collect()
    }
}

#[derive(Debug)]
struct FrameHeaderDescriptor(u8);

impl FrameHeaderDescriptor {
    fn new(byte: u8) -> Self {
        let s = Self(byte);
        assert!(
            !s.unused_flag(),
            "Unused bit in frame header descriptor is set"
        );
        assert!(
            !s.reserved_flag(),
            "Reserved bit in frame header descriptor is set"
        );
        s
    }

    fn get_bit(&self, n: u8) -> bool {
        self.0 & (1 << n) != 0
    }

    fn frame_content_size_flag(&self) -> u8 {
        self.0 >> 6
    }

    fn frame_content_size_field_size(&self) -> u8 {
        let fcs = self.frame_content_size_flag();
        if fcs == 0 {
            self.single_segment_flag().into()
        } else {
            1 << fcs
        }
    }

    fn single_segment_flag(&self) -> bool {
        self.get_bit(5)
    }

    fn unused_flag(&self) -> bool {
        self.get_bit(4)
    }

    fn reserved_flag(&self) -> bool {
        self.get_bit(3)
    }

    fn content_checksum_flag(&self) -> bool {
        self.get_bit(2)
    }

    fn dictionary_id_flag(&self) -> u8 {
        self.0 & 0b11
    }

    fn dictionary_id_field_size(&self) -> u8 {
        let flag = self.dictionary_id_flag();
        let arr = [0, 1, 2, 4];
        arr[flag as usize]
    }
}

#[derive(Debug)]
struct WindowDescriptor(u8);

impl WindowDescriptor {
    fn new(byte: u8) -> Self {
        Self(byte)
    }

    fn to_window_size(&self) -> u64 {
        let exponent = self.0 >> 3;
        let mantissa = self.0 & 0b111;

        let window_log = 10 + exponent;
        let window_base = 1u64 << window_log;
        let window_add = (window_base / 8) * mantissa as u64;
        window_base + window_add
    }
}

#[derive(Debug)]
struct FrameHeader {
    frame_header_descriptor: FrameHeaderDescriptor,
    window_size: u64,
    dictionary_id: u32,
    frame_content_size: u64,
    len: usize,
}

impl FrameHeader {
    fn from_bytes(bytes: &[u8]) -> Res<Self> {
        let fhd = FrameHeaderDescriptor::new(bytes[0]);
        let mut index = 1usize;

        let window_descriptor = if fhd.single_segment_flag() {
            None
        } else {
            index += 1;
            Some(WindowDescriptor::new(bytes[1]))
        };

        let did_field_size = fhd.dictionary_id_field_size();
        let dictionary_id = Self::parse_dictionary_id(&bytes[index..], did_field_size)?;
        index += did_field_size as usize;

        let fcs_field_size = fhd.frame_content_size_field_size();
        let frame_content_size = Self::parse_frame_content_size(&bytes[index..], fcs_field_size)?;
        index += fcs_field_size as usize;

        let window_size = window_descriptor
            .map(|wd| wd.to_window_size())
            .unwrap_or(frame_content_size);

        let frame_header = Self {
            frame_header_descriptor: fhd,
            window_size,
            dictionary_id,
            frame_content_size,
            len: index,
        };

        Ok(frame_header)
    }

    fn parse_dictionary_id(bytes: &[u8], field_size: u8) -> Res<u32> {
        let did = match field_size {
            0 => 0,
            1 => u8::from_le_bytes(bytes[..1].try_into()?) as u32,
            2 => u16::from_le_bytes(bytes[..2].try_into()?) as u32,
            4 => u32::from_le_bytes(bytes[..4].try_into()?),
            _ => return Err("Invalid dictionary id field size".into()),
        };
        Ok(did)
    }

    fn parse_frame_content_size(bytes: &[u8], field_size: u8) -> Res<u64> {
        let fcs = match field_size {
            0 => 0,
            1 => u8::from_le_bytes(bytes[..1].try_into()?) as u64,
            2 => u16::from_le_bytes(bytes[..2].try_into()?) as u64 + 256,
            4 => u32::from_le_bytes(bytes[..4].try_into()?) as u64,
            8 => u64::from_le_bytes(bytes[..8].try_into()?),
            _ => return Err("Invalid frame content size field size".into()),
        };
        Ok(fcs)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum BlockType {
    Raw,
    Rle,
    Compressed,
    Reserved,
}

#[derive(Debug)]
struct BlockHeader([u8; 3]);

impl BlockHeader {
    fn from_bytes(bytes: [u8; 3]) -> Self {
        Self(bytes)
    }

    fn is_last_block(&self) -> bool {
        let flag = self.0[0] & 1;
        flag != 0
    }

    fn block_type(&self) -> BlockType {
        let flag = (self.0[0] >> 1) & 0b11;
        match flag {
            0 => BlockType::Raw,
            1 => BlockType::Rle,
            2 => BlockType::Compressed,
            3 => BlockType::Reserved,
            _ => panic!("Impossible block type"),
        }
    }

    fn block_size(&self) -> u32 {
        let [a, b, c] = self.0;
        let a = a >> 3;
        (c as u32) << 16 | (b as u32) << 8 | (a as u32)
    }
}

#[derive(Debug)]
struct Block {
    block_header: BlockHeader,
    block_content: Vec<u8>,
}

impl Block {
    fn from_bytes(bytes: &[u8]) -> Res<Self> {
        let block_header = BlockHeader::from_bytes(bytes[..3].try_into()?);

        if block_header.block_type() == BlockType::Rle {
            let block_content = vec![bytes[3]];
            return Ok(Self {
                block_header,
                block_content,
            });
        }

        let block_content = bytes[3..3 + block_header.block_size() as usize]
            .iter()
            .map(|b| *b)
            .collect();

        Ok(Self {
            block_header,
            block_content,
        })
    }

    fn len(&self) -> usize {
        3 + self.block_content.len()
    }

    fn decode(self) -> Vec<u8> {
        match self.block_header.block_type() {
            BlockType::Raw => self.block_content,
            BlockType::Rle => vec![self.block_content[0]; self.block_header.block_size() as usize],
            BlockType::Reserved => panic!("Impossible reserved block type"),
            BlockType::Compressed => todo!(),
        }
    }
}

pub struct Zstd {
    frames: Vec<Frame>,
}

impl Zstd {
    pub fn from_bytes(bytes: Vec<u8>) -> Res<Self> {
        let mut frames = vec![];
        let mut bytes: &[u8] = &bytes;

        while !bytes.is_empty() {
            let frame = Frame::from_bytes(&bytes)?;
            bytes = &bytes[frame.len()..];
            frames.push(frame);
        }

        Ok(Self { frames })
    }

    pub fn encode(bytes: Vec<u8>) -> Vec<u8> {
        todo!();
    }

    pub fn decode(self) -> Vec<u8> {
        self.frames
            .into_iter()
            .map(|frame| frame.decode())
            .flatten()
            .collect()
    }
}
