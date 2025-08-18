use log::debug;

use crate::Res;

#[derive(Debug)]
pub struct CompressedBlock {
    literals_section: LiteralsSection,
    sequences_section: SequencesSection,
}

impl CompressedBlock {
    pub fn from_bytes(bytes: &[u8]) -> Res<Self> {
        let literals_section = LiteralsSection::from_bytes(bytes);
        todo!();
    }
}

#[derive(Debug)]
enum Streams {
    One(Vec<u8>),
    Four([Vec<u8>; 4]),
}

impl Streams {
    fn len(&self) -> usize {
        match self {
            Streams::One(s) => s.len(),
            Streams::Four(streams) => streams.iter().map(|s| s.len()).sum(),
        }
    }
}

#[derive(Debug)]
struct LiteralsSection {
    literals_section_header: LiteralsSectionHeader,
    huffman_tree_description: Option<()>,
    jump_table: Option<[u16; 3]>,
    streams: Streams,
}

impl LiteralsSection {
    pub fn from_bytes(mut bytes: &[u8]) -> Res<Self> {
        let literals_block_type = LiteralsSectionHeader::literals_block_type(bytes[0]);
        let lsh = LiteralsSectionHeader::from_bytes(bytes)?;
        bytes = &bytes[lsh.header_len..];

        match literals_block_type {
            LiteralsBlockType::RawLiteralsBlock => {
                let stream = bytes[..lsh.regenerated_size as usize].into();
                Ok(Self {
                    literals_section_header: lsh,
                    huffman_tree_description: None,
                    jump_table: None,
                    streams: Streams::One(stream),
                })
            }
            LiteralsBlockType::RleLiteralsBlock => Ok(Self {
                literals_section_header: lsh,
                huffman_tree_description: None,
                jump_table: None,
                streams: Streams::One(bytes[..1].into()),
            }),
            LiteralsBlockType::CompressedLiteralsBlock => {
                todo!();
            }
            LiteralsBlockType::TreelessLiteralsBlock => {
                todo!();
            }
        }
    }

    fn len(&self) -> usize {
        if self.huffman_tree_description.is_some() {
            todo!();
        }
        let jump_table_size = if self.jump_table.is_some() { 6 } else { 0 };
        self.literals_section_header.header_len + jump_table_size + self.streams.len()
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LiteralsBlockType {
    RawLiteralsBlock,
    RleLiteralsBlock,
    CompressedLiteralsBlock,
    TreelessLiteralsBlock,
}

#[derive(Debug)]
struct LiteralsSectionHeader {
    regenerated_size: u32,
    compressed_size: Option<u32>,
    header_len: usize,
}

impl LiteralsSectionHeader {
    fn new(regenerated_size: u32, compressed_size: Option<u32>, header_len: usize) -> Self {
        Self {
            regenerated_size,
            compressed_size,
            header_len,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Res<Self> {
        let s = Self::_from_bytes(bytes);

        debug!("literal_section_header {:x?}", &bytes[..s.header_len]);
        debug!(
            "{:?}, size_format {:?}, {:?}",
            Self::literals_block_type(bytes[0]),
            Self::size_format(bytes[0]),
            s,
        );

        Ok(s)
    }

    fn literals_block_type(byte: u8) -> LiteralsBlockType {
        match byte & 0b11 {
            0 => LiteralsBlockType::RawLiteralsBlock,
            1 => LiteralsBlockType::RleLiteralsBlock,
            2 => LiteralsBlockType::CompressedLiteralsBlock,
            3 => LiteralsBlockType::TreelessLiteralsBlock,
            _ => panic!("impossible literals_block_type"),
        }
    }

    fn size_format(byte: u8) -> u8 {
        (byte >> 2) & 0b11
    }

    fn is_one_stream(byte: u8) -> bool {
        let t = Self::literals_block_type(byte);
        t == LiteralsBlockType::RawLiteralsBlock
            || t == LiteralsBlockType::RleLiteralsBlock
            || Self::size_format(byte) == 0b00
    }

    fn _from_bytes(bytes: &[u8]) -> Self {
        let literals_block_type = Self::literals_block_type(bytes[0]);
        let size_format = Self::size_format(bytes[0]);

        let bytes = bytes.iter().map(|b| *b as u32).collect::<Vec<_>>();

        match literals_block_type {
            LiteralsBlockType::RawLiteralsBlock | LiteralsBlockType::RleLiteralsBlock => {
                match size_format {
                    0b00 | 0b10 => Self::new(bytes[0] >> 3, None, 1),
                    0b01 => Self::new((bytes[0] >> 4) + (bytes[1] << 4), None, 2),
                    0b11 => Self::new(
                        (bytes[0] >> 4) + (bytes[1] << 4) + (bytes[2] << 12),
                        None,
                        3,
                    ),
                    _ => panic!("impossible size_format"),
                }
            }
            LiteralsBlockType::CompressedLiteralsBlock
            | LiteralsBlockType::TreelessLiteralsBlock => match size_format {
                0b00 | 0b01 => Self::new(
                    (bytes[0] >> 4) + ((bytes[1] & 0b111111) << 4),
                    Some((bytes[1] >> 6) + (bytes[2] << 2)),
                    3,
                ),
                0b10 => Self::new(
                    (bytes[0] >> 4) + (bytes[1] << 4) + ((bytes[2] & 0b11) << 12),
                    Some((bytes[2] >> 2) + (bytes[3] << 6)),
                    4,
                ),
                0b11 => Self::new(
                    (bytes[0] >> 4) + (bytes[1] << 4) + ((bytes[2] & 0b111111) << 12),
                    Some((bytes[2] >> 6) + (bytes[3] << 2) + (bytes[4] << 10)),
                    5,
                ),
                _ => panic!("impossible size_format"),
            },
        }
    }
}

#[derive(Debug)]
struct SequencesSection {}
