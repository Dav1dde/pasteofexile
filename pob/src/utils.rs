use crate::{Error, Result};
use flate2::bufread::ZlibDecoder;
use std::io::Read;

pub fn decompress(data: &str) -> Result<String> {
    let data = decode(data)?;
    deflate(&data)
}

fn decode(data: &str) -> Result<Vec<u8>> {
    base64::decode_config(data.trim(), base64::URL_SAFE).map_err(Error::Base64Decode)
}

fn deflate(inp: &[u8]) -> Result<String> {
    let mut deflater = ZlibDecoder::new(inp);
    let mut buf = Vec::new();
    deflater.read_to_end(&mut buf).map_err(Error::Deflate)?;

    match String::from_utf8(buf) {
        Ok(s) => Ok(s),
        Err(e) => {
            use encoding::{all::WINDOWS_1252, DecoderTrap, Encoding};
            WINDOWS_1252
                .decode(&e.into_bytes(), DecoderTrap::Strict)
                .map_err(Error::StringDecode)
        }
    }
}
