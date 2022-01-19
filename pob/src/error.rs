use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unable to base64 decode input: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("failed to deflate/decompress input: {0}")]
    Deflate(#[from] std::io::Error),

    #[error("failed to parse input XML: {0:?}")]
    ParseXml(#[from] quick_xml::de::DeError),
}
