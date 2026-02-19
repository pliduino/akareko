use std::fmt::{Display, Formatter};

use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::{Engine as _, engine::general_purpose::STANDARD_NO_PAD};
use ed25519_dalek::{SigningKey, ed25519::signature::SignerMut};
use rand::rngs::OsRng;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use zeroize::ZeroizeOnDrop;

use crate::errors::Base64Error;

#[derive(Serialize, Deserialize, Debug, Clone, ZeroizeOnDrop)]
#[serde(transparent)]
pub struct PrivateKey(#[serde(with = "serde_bytes")] [u8; 32]);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, byteable_derive::Byteable)]
#[serde(transparent)]
pub struct PublicKey(#[serde(with = "serde_bytes")] [u8; 32]);

#[derive(Debug, Clone, Serialize, Deserialize, byteable_derive::Byteable)]
pub struct Signature(#[serde(with = "serde_bytes")] [u8; 64]);

impl Signature {
    pub fn empty() -> Self {
        Signature([0u8; 64])
    }

    pub fn to_inner(self) -> [u8; 64] {
        self.0
    }

    pub fn as_base64(&self) -> String {
        STANDARD_NO_PAD.encode(&self.0)
    }

    pub fn as_base64_url(&self) -> String {
        BASE64_URL_SAFE_NO_PAD.encode(&self.0)
    }

    pub fn from_base64(base64: &str) -> Result<Self, Base64Error> {
        let bytes = STANDARD_NO_PAD.decode(base64)?;

        match bytes.try_into() {
            Ok(hash) => Ok(Signature(hash)),
            Err(b) => Err(Base64Error::InvalidLength {
                expected: 64,
                actual: b.len(),
            }),
        }
    }
}

impl PrivateKey {
    pub fn new() -> Self {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);

        PrivateKey(signing_key.to_bytes())
    }

    pub fn sign(&self, msg: &[u8]) -> Signature {
        let mut signing_key = ed25519_dalek::SigningKey::from_bytes(&self.0);
        let signature = signing_key.sign(msg);

        Signature(signature.to_bytes())
    }

    pub fn public_key(&self) -> PublicKey {
        let signing_key = ed25519_dalek::SigningKey::from(&self.0);
        PublicKey(signing_key.verifying_key().to_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_base64(&self) -> String {
        STANDARD_NO_PAD.encode(&self.0)
    }

    pub fn from_base64(base64: &str) -> Result<Self, Base64Error> {
        let bytes = STANDARD_NO_PAD.decode(base64)?;

        match bytes.try_into() {
            Ok(hash) => Ok(PrivateKey(hash)),
            Err(b) => Err(Base64Error::InvalidLength {
                expected: 32,
                actual: b.len(),
            }),
        }
    }
}

impl PublicKey {
    pub fn verify(&self, msg: &[u8], signature: &Signature) -> bool {
        let signature = ed25519_dalek::Signature::from_bytes(&signature.0);
        let verifying_key = match ed25519_dalek::VerifyingKey::from_bytes(&self.0) {
            Ok(key) => key,
            Err(_) => return false,
        };
        verifying_key.verify_strict(msg, &signature).is_ok()
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_base64(&self) -> String {
        STANDARD_NO_PAD.encode(&self.0)
    }

    pub fn from_base64(base64: &str) -> Result<Self, Base64Error> {
        let bytes = STANDARD_NO_PAD.decode(base64)?;

        match bytes.try_into() {
            Ok(hash) => Ok(PublicKey(hash)),
            Err(b) => Err(Base64Error::InvalidLength {
                expected: 32,
                actual: b.len(),
            }),
        }
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        for i in self.0 {
            str.push_str(&format!("{:02x}", i));
        }
        write!(f, "{}", str)
    }
}

pub trait Signable {
    fn sign(&self, private_key: &PrivateKey) -> Signature;
    fn verify(&self, public_key: &PublicKey, signature: &Signature) -> bool;
}
