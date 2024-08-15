use std::io;

// compress binary data using zstd
#[allow(dead_code)]
pub fn compress(data: &Vec<u8>) -> Vec<u8> {
    zstd::bulk::compress(&data[..], 3).unwrap()
}

// decompress binary data using zstd
#[allow(dead_code)]
pub fn decompress(data: &Vec<u8>) -> io::Result<Vec<u8>> {
    zstd::stream::decode_all(&data[..])
}