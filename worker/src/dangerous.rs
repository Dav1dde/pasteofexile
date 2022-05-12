use crate::crypto;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt;

#[derive(Debug)]
pub enum DangerousError {
    BadEncoding,
    BadSignature,
    Serialize,
    Deserialize,
    Crypto,
}

impl fmt::Display for DangerousError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadEncoding => write!(f, "lol"),
            Self::BadSignature => write!(f, "lol2"),
            Self::Serialize => write!(f, "lol3"),
            Self::Deserialize => write!(f, "lolxx"),
            Self::Crypto => write!(f, "lol3"),
        }
    }
}

impl std::error::Error for DangerousError {}

type Result<T> = std::result::Result<T, DangerousError>;

pub struct Dangerous {
    secret: Vec<u8>,
}

impl Dangerous {
    pub fn new(secret: Vec<u8>) -> Self {
        Self { secret }
    }

    pub async fn sign<T: Serialize>(&self, data: &T) -> Result<String> {
        let mut payload = serde_json::to_vec(data).map_err(|_| DangerousError::Serialize)?;

        let signature = crypto::sign_hmac_256(&self.secret, &mut payload)
            .await
            .map_err(|_| DangerousError::Crypto)?;

        let mut result = base64::encode_config(payload, base64::URL_SAFE_NO_PAD);
        result.push('.');
        result.push_str(&base64::encode_config(signature, base64::URL_SAFE_NO_PAD));

        Ok(result)
    }

    #[allow(dead_code)] // TODO remove me
    pub async fn verify<T: DeserializeOwned>(&self, data: &str) -> Result<T> {
        let (payload, signature) = data.rsplit_once('.').unwrap();

        let mut signature = base64::decode_config(signature, base64::URL_SAFE_NO_PAD)
            .map_err(|_| DangerousError::BadEncoding)?;
        let mut payload = base64::decode_config(payload, base64::URL_SAFE_NO_PAD)
            .map_err(|_| DangerousError::BadEncoding)?;

        let verified = crypto::verify_hmac_256(&self.secret, &mut signature, &mut payload)
            .await
            .map_err(|_| DangerousError::Crypto)?;

        if !verified {
            return Err(DangerousError::BadSignature);
        }

        serde_json::from_slice(&payload).map_err(|_| DangerousError::Deserialize)
    }
}
