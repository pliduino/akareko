mod keys;

use std::fmt::Display;

use base64::{Engine as _, engine::general_purpose::STANDARD_NO_PAD};
use serde::{Deserialize, Serialize};
use sha2::Digest;

pub use keys::{PrivateKey, PublicKey, Signable, Signature};

#[derive(
    Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, byteable_derive::Byteable,
)]
#[serde(transparent)]
pub struct Hash([u8; 32]);

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
    pub fn new(hash: [u8; 32]) -> Self {
        Hash(hash)
    }

    pub fn digest(bytes: &[u8]) -> Self {
        let hash = sha2::Sha256::digest(bytes);
        Hash(hash.into())
    }

    pub fn inner(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn as_base64(&self) -> String {
        STANDARD_NO_PAD.encode(&self.0)
    }

    pub fn from_base64(base64: &str) -> Result<Self, ()> {
        let bytes = match STANDARD_NO_PAD.decode(base64) {
            Ok(bytes) => bytes,
            Err(_) => return Err(()),
        };

        match bytes.try_into() {
            Ok(hash) => Ok(Hash(hash)),
            Err(_) => Err(()), //TODO: Add proper error
        }
    }
}
