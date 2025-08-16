use std::fs;

use zstd::{Res, Zstd};

fn _encode(path: &str) -> Res<()> {
    let output_path = path.to_string() + ".zst";

    let input_bytes = fs::read(path)?;
    let output_bytes = Zstd::encode(input_bytes);

    if fs::exists(&output_path)? {
        println!("Overwriting output file...");
    }
    fs::write(output_path, output_bytes)?;

    Ok(())
}

fn decode(path: &str) -> Res<()> {
    if !path.ends_with(".zst") {
        return Err("File name to decode should end with .zst".into());
    }
    let output_path = &path[..path.len() - 4];

    let input_bytes = fs::read(path)?;
    let output_bytes = Zstd::from_bytes(input_bytes)?.decode();

    if fs::exists(output_path)? {
        println!("Overwriting output file...");
    }
    fs::write(output_path, output_bytes)?;

    Ok(())
}

fn main() -> Res<()> {
    // let path = "tests/hello.txt";
    // _encode(path)?;

    let path = "tests/hello.txt.zst";
    decode(path)?;

    Ok(())
}
