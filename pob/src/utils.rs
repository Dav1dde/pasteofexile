use zune_inflate::DeflateDecoder;

use crate::{Error, Result};

pub fn decompress(data: &str) -> Result<String> {
    let data = decode(data)?;
    deflate(&data)
}

fn decode(data: &str) -> Result<Vec<u8>> {
    base64::decode_config(data.trim(), base64::URL_SAFE).map_err(Error::Base64Decode)
}

fn deflate(inp: &[u8]) -> Result<String> {
    let mut deflater = DeflateDecoder::new(inp);
    let buf = deflater
        .decode_zlib()
        .map_err(|e| format!("{:?}", e.error))
        .map_err(Error::Deflate)?;

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
