use log::debug;

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

    pub fn get_bits(&mut self, mut n: u8) -> u64 {
        if n == 0 {
            return 0;
        }

        if n > 64 {
            panic!("too many bytes to read: {} bytes", n);
        }

        if self.current_bit == 0 {
            let skipping_bits = self.bytes[0].leading_zeros() + 1;
            debug!("skipping {} bits", skipping_bits);
            self.current_bit += skipping_bits as usize;
        }

        let mut i = self.current_bit / 8;
        let used_bits_len = self.current_bit as u8 % 8;
        self.current_bit += n as usize;
        let unused_bits_len = 8 - used_bits_len;

        if n <= unused_bits_len {
            return Self::get_bits_range(self.bytes[i], used_bits_len, n) as u64;
        }

        let mut result = Self::get_bits_range(self.bytes[i], used_bits_len, unused_bits_len) as u64;
        n -= unused_bits_len;
        while n > 8 {
            result = (result << 8) + self.bytes[i] as u64;
            i += 1;
            n -= 8;
        }
        let last_bits = Self::get_bits_range(self.bytes[i + 1], 0, n) as u64;
        (result << n) + last_bits
    }

    fn get_bits_range(byte: u8, start: u8, n: u8) -> u8 {
        if start == 0 && n == 8 {
            return byte;
        }
        if start > 8 || n > 8 {
            panic!("start or n too big");
        }

        let mask = (1 << n) - 1;
        (byte >> (8 - start - n)) & mask
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_bits_range() {
        let byte = 0b01101001;
        let actual = Bitstream::get_bits_range(byte, 2, 5);
        let expected = 0b10100;
        println!("{:b} {:b}", actual, expected);
        assert_eq!(actual, expected);
    }

    #[test]
    fn get_bits() {
        let bytes = 0x1234567801_u64.to_le_bytes();
        println!("{:02x?}", bytes);
        let mut bs = Bitstream::new(bytes.to_vec());
        let actual = bs.get_bits(8);
        let expected = 0x78;
        println!("{:08b} {:08b}", actual, expected);
        assert_eq!(actual, expected);

        let actual = bs.get_bits(4);
        let expected = 0x5;
        println!("{:04b} {:04b}", actual, expected);
        assert_eq!(actual, expected);

        let actual = bs.get_bits(8);
        let expected = 0x63;
        println!("{:08b} {:08b}", actual, expected);
        assert_eq!(actual, expected);
    }

    #[test]
    fn get_bits_more_than_8() {
        let bytes = 0x1234567801_u64.to_le_bytes();
        println!("{:02x?}", bytes);
        let mut bs = Bitstream::new(bytes.to_vec());
        let actual = bs.get_bits(16);
        let expected = 0x7856;
        println!("{:08b} {:08b}", actual, expected);
        assert_eq!(actual, expected);

        let actual = bs.get_bits(4);
        let expected = 0x3;
        println!("{:04b} {:04b}", actual, expected);
        assert_eq!(actual, expected);

        let actual = bs.get_bits(12);
        let expected = 0x412;
        println!("{:08b} {:08b}", actual, expected);
        assert_eq!(actual, expected);
    }

    #[test]
    fn thousand_a() {
        let bytes = vec![0x05, 0x80, 0x2b, 0xe3];
        println!("{:02x?}", bytes);
        let mut bs = Bitstream::new(bytes);
        let actual = bs.get_bits(6);
        let expected = 24;
        println!("{:06b} {:06b}", actual, expected);
        assert_eq!(actual, expected);

        let actual = bs.get_bits(6);
        let expected = 0;
        println!("{:06b} {:06b}", actual, expected);
        assert_eq!(actual, expected);

        let actual = bs.get_bits(5);
        let expected = 21;
        println!("{:05b} {:05b}", actual, expected);
        assert_eq!(actual, expected);
    }
}
