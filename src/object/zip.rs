use std::io::{Read, Write};

use anyhow::Result;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;
    Ok(decompressed_data)
}

pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let data = b"Hello, world\0!";
        let compressed_data = compress(data).unwrap();
        let decompressed_data = decompress(&compressed_data).unwrap();
        assert_eq!(data, decompressed_data.as_slice());
    }

    #[test]
    fn test_decompress_compress() {
        let raw_data = b"Hello, world\0!";
        let data = compress(raw_data).unwrap();

        let decompressed_data = decompress(&data).unwrap();
        let compressed_data = compress(&decompressed_data).unwrap();
        assert_eq!(data, compressed_data.as_slice());
    }
}
