use std::io::{self, Read, Write};

use lz4::EncoderBuilder;



// compress binary data using lz4
#[allow(dead_code)]
pub fn compress(data: &Vec<u8>) -> Vec<u8> {
    let mut encoder = EncoderBuilder::new()
        .level(10)
        .build(Vec::new()).unwrap();

    encoder.write_all(&data).unwrap();
    encoder.finish().0
}

// decompress binary data using lz4
#[allow(dead_code)]
pub fn decompress(data: &Vec<u8>) -> io::Result<Vec<u8>> {
    let mut decoder = lz4::Decoder::new(&data[..])?;
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}