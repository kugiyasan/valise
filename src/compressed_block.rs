use log::debug;

use crate::{
    Res,
    bitstream::Bitstream,
    fse::{FseDecoder, FseDecodingTable},
};

#[derive(Debug)]
pub struct CompressedBlock {
    literals_section: LiteralsSection,
    sequences_section: SequencesSection,
}

impl CompressedBlock {
    pub fn from_bytes(mut bytes: &[u8]) -> Res<Self> {
        let literals_section = LiteralsSection::from_bytes(bytes)?;
        debug!("LiteralsSection {:02x?}", &bytes[..literals_section.len()]);
        bytes = &bytes[literals_section.len()..];

        let sequences_section = SequencesSection::from_bytes(bytes)?;
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

        debug!("literal_section_header {:02x?}", &bytes[..s.header_len]);
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
struct SequencesSection {
    sequences_section_header: SequencesSectionHeader,
}

impl SequencesSection {
    fn from_bytes(mut bytes: &[u8]) -> Res<Self> {
        let sequences_section_header = SequencesSectionHeader::from_bytes(bytes)?;
        debug!(
            "sequences_section_header {:02x?}",
            &bytes[..sequences_section_header.len()]
        );
        debug!("{:?}", sequences_section_header);
        bytes = &bytes[sequences_section_header.len()..];

        if sequences_section_header
            .symbol_compression_modes
            .literal_lengths_mode()
            != CompressionMode::PredefinedMode
        {
            todo!();
        }

        if sequences_section_header
            .symbol_compression_modes
            .offsets_mode()
            != CompressionMode::PredefinedMode
        {
            todo!();
        }

        if sequences_section_header
            .symbol_compression_modes
            .match_lengths_mode()
            != CompressionMode::PredefinedMode
        {
            todo!();
        }

        let ll_table = FseDecodingTable::literals_length_default_distribution();
        let ml_table = FseDecodingTable::match_lengths_default_distribution();
        let of_table = FseDecodingTable::offset_codes_default_distribution();

        let mut bs = Bitstream::new(bytes.iter().rev().map(|b| *b).collect());
        debug!("{:02x?}", bytes.iter().rev().map(|b| *b).collect::<Vec<_>>());

        let ll_init_state = bs.get_bits(ll_table.accuracy_log());
        let ml_init_state = bs.get_bits(ml_table.accuracy_log());
        let of_init_state = bs.get_bits(of_table.accuracy_log());

        let ll = FseDecoder::new(ll_table, ll_init_state);
        let ml = FseDecoder::new(ml_table, ml_init_state);
        let of = FseDecoder::new(of_table, of_init_state);
        debug!(
            "init states: {}, {}, {}",
            ll_init_state, ml_init_state, of_init_state
        );

        for _ in 0..sequences_section_header.number_of_sequences {}

        todo!("{:02x?}", bytes);
    }

    fn literals_length_code(literals_length_code: u8) -> (u32, u8) {
        match literals_length_code {
            0..=15 => (literals_length_code as u32, 0),
            16 => (16, 1),
            17 => (18, 1),
            18 => (20, 1),
            19 => (22, 1),
            20 => (24, 2),
            21 => (28, 2),
            22 => (32, 3),
            23 => (40, 3),
            24 => (48, 4),
            25 => (64, 6),
            26 => (128, 7),
            27 => (256, 8),
            28 => (512, 9),
            29 => (1024, 10),
            30 => (2048, 11),
            31 => (4096, 12),
            32 => (8192, 13),
            33 => (16384, 14),
            34 => (32768, 15),
            35 => (65536, 16),
            _ => panic!("impossible literals_length_code"),
        }
    }

    fn match_length_code(match_length_code: u8) -> (u32, u8) {
        match match_length_code {
            0..=31 => (match_length_code as u32 + 3, 0),
            32 => (35, 1),
            33 => (37, 1),
            34 => (39, 1),
            35 => (41, 1),
            36 => (43, 2),
            37 => (47, 2),
            38 => (51, 3),
            39 => (59, 3),
            40 => (67, 4),
            41 => (83, 4),
            42 => (99, 5),
            43 => (131, 7),
            44 => (259, 8),
            45 => (515, 9),
            46 => (1027, 10),
            47 => (2051, 11),
            48 => (4099, 12),
            49 => (8195, 13),
            50 => (16387, 14),
            51 => (32771, 15),
            52 => (65539, 16),
            _ => panic!("impossible literals_length_code"),
        }
    }
}

#[derive(Debug)]
struct SequencesSectionHeader {
    number_of_sequences: u16,
    number_of_sequences_size: usize,
    symbol_compression_modes: SymbolCompressionModes,
}

impl SequencesSectionHeader {
    fn from_bytes(mut bytes: &[u8]) -> Res<Self> {
        let number_of_sequences = Self::number_of_sequences(bytes);
        let number_of_sequences_size = Self::number_of_sequences_size(bytes[0]);
        bytes = &bytes[number_of_sequences_size..];
        let symbol_compression_modes = SymbolCompressionModes::new(bytes[0]);
        Ok(Self {
            number_of_sequences,
            number_of_sequences_size,
            symbol_compression_modes,
        })
    }

    fn number_of_sequences(bytes: &[u8]) -> u16 {
        if bytes[0] == 0 {
            0
        } else if bytes[0] < 128 {
            bytes[0].into()
        } else if bytes[0] < 255 {
            ((bytes[0] as u16 - 128) << 8) + bytes[1] as u16
        } else {
            (bytes[1] as u16) << 8 + bytes[2] as u16
        }
    }

    fn number_of_sequences_size(byte: u8) -> usize {
        if byte == 0 {
            0
        } else if byte < 128 {
            1
        } else if byte < 255 {
            2
        } else {
            3
        }
    }

    fn len(&self) -> usize {
        self.number_of_sequences_size + 1
    }
}

#[derive(Debug, PartialEq, Eq)]
enum CompressionMode {
    PredefinedMode,
    RleMode,
    FseCompressedMode,
    RepeatMode,
}

#[derive(Debug)]
struct SymbolCompressionModes(u8);

impl SymbolCompressionModes {
    fn new(byte: u8) -> Self {
        let s = Self(byte);
        assert_eq!(s.reserved(), 0);
        debug!(
            "literal lengths {:?}, offsets {:?} match lengths {:?}",
            s.literal_lengths_mode(),
            s.offsets_mode(),
            s.match_lengths_mode()
        );
        s
    }

    fn get_2_bits(&self, n: u8) -> u8 {
        (self.0 >> n) & 0b11
    }

    fn get_compression_mode(&self, n: u8) -> CompressionMode {
        match self.get_2_bits(n) {
            0 => CompressionMode::PredefinedMode,
            1 => CompressionMode::RleMode,
            2 => CompressionMode::FseCompressedMode,
            3 => CompressionMode::RepeatMode,
            _ => panic!("impossible compression mode"),
        }
    }

    fn literal_lengths_mode(&self) -> CompressionMode {
        self.get_compression_mode(6)
    }

    fn offsets_mode(&self) -> CompressionMode {
        self.get_compression_mode(4)
    }

    fn match_lengths_mode(&self) -> CompressionMode {
        self.get_compression_mode(2)
    }

    fn reserved(&self) -> u8 {
        self.get_2_bits(0)
    }
}
