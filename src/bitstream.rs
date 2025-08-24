pub struct Bitstream {
    bytes: Vec<u8>,
    current_bit: usize,
}

impl Bitstream {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            current_bit: 0,
        }
    }

    pub fn get_bits(&mut self, n: u8) -> u8 {
        if n > 8 {
            todo!();
        }

        let i = self.current_bit / 8;
        let used_bits_len = self.current_bit as u8 % 8;
        self.current_bit += n as usize;
        let unused_bits_len = 8 - used_bits_len;

        if n <= unused_bits_len {
            return Self::get_bits_range(self.bytes[i], used_bits_len + n, n);
        }

        let bits0 = Self::get_bits_range(self.bytes[i], 8, unused_bits_len as u8);
        let remaining_bits_len = n - unused_bits_len;
        let bits1 = Self::get_bits_range(
            self.bytes[i + 1],
            8 - remaining_bits_len,
            remaining_bits_len,
        );

        (bits0 << unused_bits_len) + bits1
    }

    fn get_bits_range(byte: u8, start: u8, n: u8) -> u8 {
        if start == 8 && n == 8 {
            return byte;
        }
        if start > 8 || n > 8 {
            panic!("start or n too big");
        }

        let mask = (1 << n) - 1;
        (byte >> (start - n)) & mask
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_bits_range() {
        let byte = 0b01101001;
        let actual = Bitstream::get_bits_range(byte, 6, 5);
        let expected = 0b10100;
        println!("{:b} {:b}", actual, expected);
        assert_eq!(actual, expected);
    }

    #[test]
    fn get_bits() {
        let bytes = 0b0001_0010_0011_0100_0101_0110_0111u32.to_le_bytes();
        println!("{:02x?}", bytes);
        let mut bs = Bitstream::new(bytes.to_vec());
        let actual = bs.get_bits(8);
        let expected = 0b0110_0111;
        println!("{:08b} {:08b}", actual, expected);
        assert_eq!(actual, expected);

        let actual = bs.get_bits(4);
        let expected = 0b0101;
        println!("{:04b} {:04b}", actual, expected);
        assert_eq!(actual, expected);

        let actual = bs.get_bits(8);
        let expected = 0b0100_0011;
        println!("{:08b} {:08b}", actual, expected);
        assert_eq!(actual, expected);
    }

    #[test]
    fn thousand_a() {
        let bytes = vec![0x05, 0x80, 0x2b, 0xe3];
        println!("{:02x?}", bytes);
        let mut bs = Bitstream::new(bytes);
        let actual = bs.get_bits(6);
        let expected = 5;
        println!("{:08b} {:08b}", actual, expected);
        assert_eq!(actual, expected);

        let actual = bs.get_bits(6);
        let expected = 0;
        println!("{:04b} {:04b}", actual, expected);
        assert_eq!(actual, expected);

        let actual = bs.get_bits(5);
        let expected = 24;
        println!("{:08b} {:08b}", actual, expected);
        assert_eq!(actual, expected);
    }
}
