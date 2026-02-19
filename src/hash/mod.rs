mod keys;

use std::fmt::Display;

use base64::{Engine as _, engine::general_purpose::STANDARD_NO_PAD};
use serde::{Deserialize, Serialize};
use sha2::Digest;

pub use keys::{PrivateKey, PublicKey, Signable, Signature};

use crate::errors::Base64Error;

#[derive(Debug, Clone, PartialEq, Eq, byteable_derive::Byteable)]
pub struct Hash([u8; 64]);

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = Vec::deserialize(deserializer)?;
        match bytes.try_into() {
            Ok(hash) => Ok(Hash(hash)),
            Err(b) => Err(serde::de::Error::custom(format!(
                "Invalid length, expected 64 bytes, got {}",
                b.len()
            ))),
        }
    }
}

impl std::hash::Hash for Hash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_base64())
    }
}

impl Hash {
    pub fn new(hash: [u8; 64]) -> Self {
        Hash(hash)
    }

    pub fn digest(bytes: &[u8]) -> Self {
        let hash = sha2::Sha512::digest(bytes);
        Hash(hash.into())
    }

    pub fn inner(&self) -> &[u8; 64] {
        &self.0
    }

    pub fn to_inner(&self) -> [u8; 64] {
        self.0
    }

    pub fn as_base64(&self) -> String {
        STANDARD_NO_PAD.encode(&self.0)
    }

    pub fn from_base64(base64: &str) -> Result<Self, Base64Error> {
        let bytes = STANDARD_NO_PAD.decode(base64)?;

        match bytes.try_into() {
            Ok(hash) => Ok(Hash(hash)),
            Err(b) => Err(Base64Error::InvalidLength {
                actual: b.len(),
                expected: 64,
            }), //TODO: Add proper error
        }
    }
}
