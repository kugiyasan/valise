use log::debug;

use crate::Res;
use crate::compressed_block::CompressedBlock;

#[derive(Debug, PartialEq, Eq)]
enum BlockType {
    Raw,
    Rle,
    Compressed,
    Reserved,
}

#[derive(Debug)]
pub struct BlockHeader([u8; 3]);

impl BlockHeader {
    fn from_bytes(bytes: [u8; 3]) -> Self {
        let s = Self(bytes);

        debug!(
            "is_last_block {}, block_type {:?}, block_size {}",
            s.is_last_block(),
            s.block_type(),
            s.block_size()
        );
        s
    }

    pub fn is_last_block(&self) -> bool {
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
pub struct Block {
    pub block_header: BlockHeader,
    block_content: Vec<u8>,
}

impl Block {
    pub fn from_bytes(bytes: &[u8]) -> Res<Self> {
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

    pub fn len(&self) -> usize {
        3 + self.block_content.len()
    }

    pub fn decode(self) -> Vec<u8> {
        match self.block_header.block_type() {
            BlockType::Raw => self.block_content,
            BlockType::Rle => vec![self.block_content[0]; self.block_header.block_size() as usize],
            BlockType::Reserved => panic!("Impossible reserved block type"),
            BlockType::Compressed => {
                let compressed_block = CompressedBlock::from_bytes(&self.block_content);
                todo!();
            }
        }
    }
}
