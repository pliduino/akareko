use serde::{Deserialize, Serialize};
use surrealdb::types::SurrealValue;

use crate::{
    db::{Timestamp, ToBytes},
    types::{PublicKey, Signature, Topic},
};

// ==================== End Imports ====================

#[cfg(feature = "surrealdb")]
mod surreal;

// pub struct CachedSyncs {
//     pub topic: Topic,
//     pub source: PublicKey,
//     pub timestamp: Timestamp,
// }

#[derive(Debug, Clone, SurrealValue, Serialize, Deserialize)]
pub struct Post {
    #[surreal(rename = "id")]
    pub signature: Signature,

    /// Who posted
    pub source: PublicKey,

    pub topic: Topic,

    pub timestamp: Timestamp,
    pub content: String,
}

impl std::hash::Hash for Post {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.signature.hash(state);
    }
}

impl Post {
    pub const TABLE_NAME: &str = "posts";

    pub fn new(
        content: String,
        timestamp: Timestamp,
        source: PublicKey,
        topic: Topic,
        signature: Signature,
    ) -> Self {
        Self {
            source,
            signature,
            topic,
            timestamp,
            content,
        }
    }

    pub fn new_signed(
        content: String,
        timestamp: Timestamp,
        topic: Topic,
        priv_key: &crate::types::PrivateKey,
    ) -> Self {
        let mut comment = Self::new(
            content,
            timestamp,
            priv_key.public_key(),
            topic,
            Signature::empty(),
        );
        comment.sign(priv_key);
        comment
    }

    fn sign_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = self.topic.inner().to_vec();
        bytes.extend(self.content.as_bytes());
        bytes.extend(self.timestamp.to_bytes());
        bytes
    }

    fn sign(&mut self, priv_key: &crate::types::PrivateKey) {
        let to_sign = self.sign_bytes();
        self.signature = priv_key.sign(&to_sign);
    }

    pub fn verify(&self) -> bool {
        let to_verify = self.sign_bytes();
        self.source.verify(&to_verify, &self.signature)
    }
}
