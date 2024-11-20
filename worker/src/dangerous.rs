use std::{fmt, time::Duration};

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    crypto,
    request_context::{Env, FromEnv},
};

#[derive(Debug)]
pub enum DangerousError {
    BadEncoding,
    BadSignature,
    Serialize,
    Deserialize,
    Crypto,
    Expired,
}

impl fmt::Display for DangerousError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadEncoding => write!(f, "Encoding Error"),
            Self::BadSignature => write!(f, "Bad Signature"),
            Self::Serialize => write!(f, "Serialization Error"),
            Self::Deserialize => write!(f, "Deserialization Error"),
            Self::Crypto => write!(f, "Crypto Error"),
            Self::Expired => write!(f, "Expired"),
        }
    }
}

impl std::error::Error for DangerousError {}

type Result<T, E = DangerousError> = std::result::Result<T, E>;

pub struct Dangerous {
    secret: Vec<u8>,
}

impl FromEnv for Dangerous {
    fn from_env(env: &Env) -> Option<Self> {
        let secret = env.var(crate::consts::ENV_SECRET_KEY)?;
        Some(Self::new(secret.into_bytes()))
    }
}

impl Dangerous {
    const SEP: char = '.';

    pub fn new(secret: Vec<u8>) -> Self {
        Self { secret }
    }

    #[tracing::instrument(skip(self))]
    pub async fn sign<T>(&self, data: &T) -> Result<String>
    where
        T: Serialize + std::fmt::Debug,
    {
        let now = worker::Date::now().as_millis();

        let mut payload = serde_json::to_string(data).map_err(|_| DangerousError::Serialize)?;
        payload.push(Self::SEP);
        payload.push_str(&encode(&now.to_le_bytes()));

        let signature = crypto::sign_hmac_256(&self.secret, payload.as_bytes())
            .await
            .map_err(|_| DangerousError::Crypto)?;

        let mut result = payload;
        result.push(Self::SEP);
        result.push_str(&encode(&signature));

        Ok(result)
    }

    #[tracing::instrument(skip(self))]
    pub async fn verify<T: DeserializeOwned>(&self, data: &str, max_age: Duration) -> Result<T> {
        let (payload, signature) = data
            .rsplit_once(Self::SEP)
            .ok_or(DangerousError::BadEncoding)?;

        let signature = decode(signature).map_err(|_| DangerousError::BadEncoding)?;
        let verified = crypto::verify_hmac_256(&self.secret, &signature, payload.as_bytes())
            .await
            .map_err(|_| DangerousError::Crypto)?;

        if !verified {
            return Err(DangerousError::BadSignature);
        }

        let (payload, timestamp) = payload
            .rsplit_once(Self::SEP)
            .ok_or(DangerousError::BadEncoding)?;

        let time = u64::from_le_bytes(
            decode(timestamp)
                .map_err(|_| DangerousError::BadEncoding)?
                .try_into()
                .map_err(|_| DangerousError::BadEncoding)?,
        );

        let now = worker::Date::now().as_millis();
        if time > now || (now - time) as u128 > max_age.as_millis() {
            return Err(DangerousError::Expired);
        }

        serde_json::from_str(payload).map_err(|_| DangerousError::Deserialize)
    }
}

fn encode(b: &[u8]) -> String {
    base64::encode_config(b, base64::URL_SAFE_NO_PAD)
}

fn decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    base64::decode_config(s, base64::URL_SAFE_NO_PAD)
}
