// https://datatracker.ietf.org/doc/html/rfc8878#name-compression-algorithm

mod bitstream;
mod block;
mod compressed_block;
mod frame;
mod fse;

use crate::frame::Frame;

pub type Res<T> = Result<T, Box<dyn std::error::Error>>;

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
