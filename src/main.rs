use clap::{Args, Parser};
use std::{ffi::OsStr, fs, path::PathBuf};

use zstd::{Res, Zstd};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    encode_or_decode: EncodeOrDecode,

    input_path: PathBuf,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct EncodeOrDecode {
    #[arg(short, long)]
    encode: bool,

    #[arg(short, long)]
    decode: bool,
}

fn main() -> Res<()> {
    let mut cli = Cli::parse();
    let input_bytes = fs::read(&cli.input_path)?;

    if cli.encode_or_decode.encode {
        let output_path = cli.input_path.to_str().unwrap().to_string() + ".zst";
        let output_bytes = Zstd::encode(input_bytes);

        if fs::exists(&output_path)? {
            println!("Overwriting output file...");
        }
        fs::write(output_path, output_bytes)?;
    } else if cli.encode_or_decode.decode {
        if cli.input_path.extension() != Some(OsStr::new("zst")) {
            return Err("File name to decode should end with .zst".into());
        }
        cli.input_path.set_extension("");
        let output_path = cli.input_path;

        let output_bytes = Zstd::from_bytes(input_bytes)?.decode();

        if fs::exists(&output_path)? {
            println!("Overwriting output file...");
        }
        fs::write(output_path, output_bytes)?;
    }

    Ok(())
}
