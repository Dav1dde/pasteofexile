use std::fmt;

use serde::{de::DeserializeOwned, Serialize};

use crate::crypto;

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
            Self::BadEncoding => write!(f, "Encoding Error"),
            Self::BadSignature => write!(f, "Bad Signature"),
            Self::Serialize => write!(f, "Serialization Error"),
            Self::Deserialize => write!(f, "Deserialization Error"),
            Self::Crypto => write!(f, "Crypto Error"),
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

    #[tracing::instrument(skip(self))]
    pub async fn sign<T: Serialize>(&self, data: &T) -> Result<String>
    where
        T: std::fmt::Debug,
    {
        let mut payload = serde_json::to_vec(data).map_err(|_| DangerousError::Serialize)?;

        let signature = crypto::sign_hmac_256(&self.secret, &mut payload)
            .await
            .map_err(|_| DangerousError::Crypto)?;

        let mut result = base64::encode_config(payload, base64::URL_SAFE_NO_PAD);
        result.push('.');
        result.push_str(&base64::encode_config(signature, base64::URL_SAFE_NO_PAD));

        Ok(result)
    }

    #[tracing::instrument(skip(self))]
    pub async fn verify<T: DeserializeOwned>(&self, data: &str) -> Result<T> {
        let (payload, signature) = data.rsplit_once('.').ok_or(DangerousError::BadEncoding)?;

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
