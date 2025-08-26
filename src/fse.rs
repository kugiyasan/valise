use std::fmt::Debug;

const LITERALS_LENGTH_DEFAULT_DISTRIBUTION: [i8; 36] = [
    4, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 2, 1, 1, 1, 1, 1,
    -1, -1, -1, -1,
];

const MATCH_LENGTHS_DEFAULT_DISTRIBUTION: [i8; 53] = [
    1, 4, 3, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, -1, -1, -1, -1, -1, -1, -1,
];

const OFFSET_CODES_DEFAULT_DISTRIBUTION: [i8; 29] = [
    1, 1, 1, 1, 1, 1, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, -1, -1, -1, -1, -1,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FseDecodingTableEntry {
    symbol: u8,
    num_bits: u8,
    baseline: u8,
}

impl FseDecodingTableEntry {
    fn new() -> Self {
        Self {
            symbol: 0,
            num_bits: 0,
            baseline: 0,
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct FseDecodingTable {
    entries: Vec<FseDecodingTableEntry>,
    accuracy_log: u8,
}

impl FseDecodingTable {
    #[cfg(test)]
    fn new(table: &[(u8, u8, u8)]) -> Self {
        let entries = table
            .iter()
            .map(|row| FseDecodingTableEntry {
                symbol: row.0,
                num_bits: row.1,
                baseline: row.2,
            })
            .collect();

        let biggest_symbol = table.iter().map(|row| row.0).max().unwrap();
        let accuracy_log = (biggest_symbol.ilog2() + 1) as u8;

        Self {
            entries,
            accuracy_log,
        }
    }

    pub fn from_distribution(distribution: &[i8]) -> Self {
        let accuracy_log = (distribution.len().ilog2() + 1) as u8;
        let table_size = 1 << accuracy_log;
        let mut symbols = vec![None; table_size];

        let mut last_index = symbols.len() - 1;
        for (i, n) in distribution.iter().enumerate() {
            if *n == -1 {
                symbols[last_index] = Some(i as u8);
                last_index -= 1;
            }
        }

        let mut position = 0;
        for (i, n) in distribution.iter().enumerate() {
            if *n == -1 {
                continue;
            }

            let mut cells_allocated = 0;
            while cells_allocated < *n {
                if symbols[position].is_none() {
                    symbols[position] = Some(i as u8);
                    cells_allocated += 1;
                }

                position += (table_size >> 1) + (table_size >> 3) + 3;
                position &= table_size - 1;
            }
        }

        let symbols = symbols.into_iter().map(|s| s.unwrap()).collect::<Vec<_>>();
        let mut entries = vec![FseDecodingTableEntry::new(); table_size];

        for symbol in 0..distribution.len() as u8 {
            let indices = symbols
                .iter()
                .enumerate()
                .filter_map(|(i, s)| if *s == symbol { Some(i) } else { None })
                .collect::<Vec<_>>();

            let probability = indices.len();
            if probability == 1 {
                let entry = FseDecodingTableEntry {
                    symbol,
                    num_bits: accuracy_log,
                    baseline: 0,
                };
                entries[indices[0]] = entry;
                continue;
            }

            let next_power_of_2 = 1 << (probability.ilog2() + 1);
            let doubles = next_power_of_2 - probability;
            let width_size = (table_size / next_power_of_2).ilog2();

            for (i, index) in indices.iter().enumerate() {
                let num_bits = if i < doubles {
                    width_size + 1
                } else {
                    width_size
                };
                let baseline = if i < doubles {
                    (probability - doubles) * (1 << width_size) + i * (1 << (width_size + 1))
                } else {
                    (i - doubles) * (1 << width_size)
                };

                let entry = FseDecodingTableEntry {
                    symbol,
                    num_bits: num_bits as u8,
                    baseline: baseline as u8,
                };
                entries[*index] = entry;
            }
        }

        // debug!("{:?}", entries);
        Self {
            entries,
            accuracy_log,
        }
    }

    pub fn literals_length_default_distribution() -> Self {
        Self::from_distribution(&LITERALS_LENGTH_DEFAULT_DISTRIBUTION)
    }

    pub fn match_lengths_default_distribution() -> Self {
        Self::from_distribution(&MATCH_LENGTHS_DEFAULT_DISTRIBUTION)
    }

    pub fn offset_codes_default_distribution() -> Self {
        Self::from_distribution(&OFFSET_CODES_DEFAULT_DISTRIBUTION)
    }

    pub fn accuracy_log(&self) -> u8 {
        self.accuracy_log
    }
}

impl Debug for FseDecodingTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FseDecodingTable")
            .field("accuracy_log", &self.accuracy_log)
            .field(
                "entries",
                &self
                    .entries
                    .iter()
                    .enumerate()
                    .map(|(i, e)| (i, e.symbol, e.num_bits, e.baseline))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

#[derive(Debug)]
pub struct FseDecoder {
    table: FseDecodingTable,
    state: u8,
}

impl FseDecoder {
    pub fn new(table: FseDecodingTable, state: u8) -> Self {
        Self { table, state }
    }

    pub fn set_state(&mut self, state: u8) {
        self.state = state % self.table.entries.len() as u8;
    }

    pub fn symbol(&self) -> u8 {
        self.table.entries[self.state as usize].symbol
    }

    pub fn num_bits(&self) -> u8 {
        self.table.entries[self.state as usize].num_bits
    }

    pub fn baseline(&self) -> u8 {
        self.table.entries[self.state as usize].baseline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED_LITERALS_LENGTH_CODE_TABLE: [(u8, u8, u8); 65] = [
        (0, 0, 0),
        (0, 4, 0),
        (0, 4, 16),
        (1, 5, 32),
        (3, 5, 0),
        (4, 5, 0),
        (6, 5, 0),
        (7, 5, 0),
        (9, 5, 0),
        (10, 5, 0),
        (12, 5, 0),
        (14, 6, 0),
        (16, 5, 0),
        (18, 5, 0),
        (19, 5, 0),
        (21, 5, 0),
        (22, 5, 0),
        (24, 5, 0),
        (25, 5, 32),
        (26, 5, 0),
        (27, 6, 0),
        (29, 6, 0),
        (31, 6, 0),
        (0, 4, 32),
        (1, 4, 0),
        (2, 5, 0),
        (4, 5, 32),
        (5, 5, 0),
        (7, 5, 32),
        (8, 5, 0),
        (10, 5, 32),
        (11, 5, 0),
        (13, 6, 0),
        (16, 5, 32),
        (17, 5, 0),
        (19, 5, 32),
        (20, 5, 0),
        (22, 5, 32),
        (23, 5, 0),
        (25, 4, 0),
        (25, 4, 16),
        (26, 5, 32),
        (28, 6, 0),
        (30, 6, 0),
        (0, 4, 48),
        (1, 4, 16),
        (2, 5, 32),
        (3, 5, 32),
        (5, 5, 32),
        (6, 5, 32),
        (8, 5, 32),
        (9, 5, 32),
        (11, 5, 32),
        (12, 5, 32),
        (15, 6, 0),
        (17, 5, 32),
        (18, 5, 32),
        (20, 5, 32),
        (21, 5, 32),
        (23, 5, 32),
        (24, 5, 32),
        (35, 6, 0),
        (34, 6, 0),
        (33, 6, 0),
        (32, 6, 0),
    ];

    const EXPECTED_MATCH_LENGTH_CODE_TABLE: [(u8, u8, u8); 65] = [
        (0, 0, 0),
        (0, 6, 0),
        (1, 4, 0),
        (2, 5, 32),
        (3, 5, 0),
        (5, 5, 0),
        (6, 5, 0),
        (8, 5, 0),
        (10, 6, 0),
        (13, 6, 0),
        (16, 6, 0),
        (19, 6, 0),
        (22, 6, 0),
        (25, 6, 0),
        (28, 6, 0),
        (31, 6, 0),
        (33, 6, 0),
        (35, 6, 0),
        (37, 6, 0),
        (39, 6, 0),
        (41, 6, 0),
        (43, 6, 0),
        (45, 6, 0),
        (1, 4, 16),
        (2, 4, 0),
        (3, 5, 32),
        (4, 5, 0),
        (6, 5, 32),
        (7, 5, 0),
        (9, 6, 0),
        (12, 6, 0),
        (15, 6, 0),
        (18, 6, 0),
        (21, 6, 0),
        (24, 6, 0),
        (27, 6, 0),
        (30, 6, 0),
        (32, 6, 0),
        (34, 6, 0),
        (36, 6, 0),
        (38, 6, 0),
        (40, 6, 0),
        (42, 6, 0),
        (44, 6, 0),
        (1, 4, 32),
        (1, 4, 48),
        (2, 4, 16),
        (4, 5, 32),
        (5, 5, 32),
        (7, 5, 32),
        (8, 5, 32),
        (11, 6, 0),
        (14, 6, 0),
        (17, 6, 0),
        (20, 6, 0),
        (23, 6, 0),
        (26, 6, 0),
        (29, 6, 0),
        (52, 6, 0),
        (51, 6, 0),
        (50, 6, 0),
        (49, 6, 0),
        (48, 6, 0),
        (47, 6, 0),
        (46, 6, 0),
    ];

    const EXPECTED_OFFSET_CODE_TABLE: [(u8, u8, u8); 33] = [
        (0, 0, 0),
        (0, 5, 0),
        (6, 4, 0),
        (9, 5, 0),
        (15, 5, 0),
        (21, 5, 0),
        (3, 5, 0),
        (7, 4, 0),
        (12, 5, 0),
        (18, 5, 0),
        (23, 5, 0),
        (5, 5, 0),
        (8, 4, 0),
        (14, 5, 0),
        (20, 5, 0),
        (2, 5, 0),
        (7, 4, 16),
        (11, 5, 0),
        (17, 5, 0),
        (22, 5, 0),
        (4, 5, 0),
        (8, 4, 16),
        (13, 5, 0),
        (19, 5, 0),
        (1, 5, 0),
        (6, 4, 16),
        (10, 5, 0),
        (16, 5, 0),
        (28, 5, 0),
        (27, 5, 0),
        (26, 5, 0),
        (25, 5, 0),
        (24, 5, 0),
    ];

    #[test]
    fn literals_length_code_table() {
        let actual = FseDecodingTable::literals_length_default_distribution();
        let expected = FseDecodingTable::new(&EXPECTED_LITERALS_LENGTH_CODE_TABLE);
        // dbg!(&actual);
        // dbg!(&expected);
        assert_eq!(&actual.entries, &expected.entries[1..]);
    }

    #[test]
    fn match_length_code_table() {
        let actual = FseDecodingTable::match_lengths_default_distribution();
        let expected = FseDecodingTable::new(&EXPECTED_MATCH_LENGTH_CODE_TABLE);
        // dbg!(&actual);
        // dbg!(&expected);
        assert_eq!(&actual.entries, &expected.entries[1..]);
    }

    #[test]
    fn offset_code_table() {
        let actual = FseDecodingTable::offset_codes_default_distribution();
        let expected = FseDecodingTable::new(&EXPECTED_OFFSET_CODE_TABLE);
        // dbg!(&actual);
        // dbg!(&expected);
        assert_eq!(&actual.entries, &expected.entries[1..]);
    }
}
