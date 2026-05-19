use base64::{Engine, prelude::BASE64_STANDARD};
use data_encoding::BASE32_NOPAD;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use skerry::skerry;
use surrealdb_types::SurrealValue;
use unicode_normalization::UnicodeNormalization;

use crate::db::user::I2PAddress;

mod byteable;
pub use byteable::{AkarekoRead, AkarekoWrite};

mod lifo;
mod serde_byteable;
pub use lifo::LiFo;

#[derive(Debug, Clone)]
pub struct SanitizedString(String);

impl SanitizedString {
    pub fn new(s: &String) -> Self {
        let normalized: String = s
            .to_lowercase() // lowercase everything
            .nfd() // decompose accents
            .filter(|c| c.is_ascii_alphanumeric())
            .collect();

        SanitizedString(normalized)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    // pub fn as_str(&self) -> &str {
    //     &self.0
    // }

    // pub fn to_inner(self) -> String {
    //     self.0
    // }
}

#[derive(Debug, Clone, SurrealValue, Serialize, Deserialize)]
#[repr(u16)]
pub enum Language {
    Japanese,
    English,
    French,
    Portuguese,
    Unknown,
}

fn i2p_b64_fix(s: &str) -> String {
    s.trim().replace('-', "+").replace('~', "/")
}

#[skerry]
pub fn b32_from_pub_b64(pub_b64: &str) -> Result<I2PAddress, e![InvalidBase64]> {
    let b64 = pub_b64
        .trim()
        .trim_end_matches(".b64.i2p")
        .trim_end_matches(".i2p");
    let fixed = i2p_b64_fix(b64);
    let decoded = BASE64_STANDARD.decode(fixed.as_bytes())?;
    let hash = Sha256::digest(&decoded);
    let b32 = BASE32_NOPAD.encode(&hash).to_lowercase();
    let b32_52 = b32.chars().take(52).collect::<String>();
    Ok(I2PAddress::new(format!("{}.b32.i2p", b32_52)))
}
