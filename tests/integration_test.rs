use std::{
    io::Write,
    process::{Command, Stdio},
};

use zstd::{Res, Zstd};

fn compress_file(input_file_content: &[u8]) -> Res<Vec<u8>> {
    let mut zstd = Command::new("zstd")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(["-e", "-"])
        .spawn()?;

    let mut stdin = zstd.stdin.take().unwrap();
    stdin.write_all(input_file_content)?;
    drop(stdin);

    let output = zstd.wait_with_output()?;
    if !output.status.success() {
        return Err(output.status.to_string().into());
    }
    Ok(output.stdout)
}

fn compression_test(expected: &[u8]) -> Res<()> {
    let compressed = compress_file(expected)?;
    let actual = Zstd::from_bytes(compressed)?.decode();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn hello_world() -> Res<()> {
    let expected = b"hello world!";
    compression_test(expected)
}

#[test]
fn thousand_a() -> Res<()> {
    let expected = b"a".repeat(1000);
    compression_test(&expected)
}
