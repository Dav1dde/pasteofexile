use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unable to base64 decode input: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("failed to string decode: {0}")]
    StringDecode(std::borrow::Cow<'static, str>),

    #[error("failed to deflate/decompress input: {0}")]
    Deflate(std::io::Error),

    #[error("failed to parse build at: {0} ({1})")]
    ParseXml(String, quick_xml::de::DeError),
}
