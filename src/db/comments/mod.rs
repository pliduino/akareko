use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};
use surrealdb::RecordId;

use crate::{
    db::{
        Index, IndexTag, Timestamp,
        user::{User, deserialize_signature_id},
    },
    hash::{Hash, PublicKey, Signature},
};

#[cfg(feature = "surrealdb")]
mod surreal;
#[cfg(feature = "surrealdb")]
pub use surreal::PostRepository;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Topic(#[serde(with = "serde_bytes")] [u8; 64]);

impl Topic {
    pub fn from_index<I: IndexTag>(index: &Index<I>) -> Self {
        Self(index.hash().inner().clone())
    }

    pub fn from_post(post: &Post) -> Self {
        Self(post.signature.clone().to_inner())
    }

    pub fn from_entry<I: IndexTag>(index: &Index<I>, enumeration: f32) -> Self {
        let mut bytes = index.hash().inner().to_vec();
        bytes.extend(enumeration.to_le_bytes());
        Self(Hash::digest(&bytes).to_inner())
    }

    pub fn inner(&self) -> &[u8; 64] {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    #[cfg_attr(
        feature = "surrealdb",
        serde(
            rename = "id",
            deserialize_with = "deserialize_signature_id",
            skip_serializing
        )
    )]
    pub signature: Signature,

    #[cfg_attr(
        feature = "surrealdb",
        serde(
            serialize_with = "serialize_pubkey_as_user_id",
            deserialize_with = "deserialize_record_id_as_pubkey",
        )
    )]
    /// Who posted
    pub source: PublicKey,

    pub topic: Topic,

    pub timestamp: Timestamp,
    pub content: String,
}

fn serialize_pubkey_as_user_id<S>(key: &PublicKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let record_id = RecordId::from_table_key(User::TABLE_NAME, key.to_base64());
    record_id.serialize(serializer)
}

fn deserialize_record_id_as_pubkey<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let id = RecordId::deserialize(deserializer)?;
    let key = id.key().to_string();
    let trimmed = key.trim_start_matches("`").trim_end_matches("`");

    PublicKey::from_base64(&trimmed).map_err(serde::de::Error::custom)
}

impl Post {
    const TABLE_NAME: &str = "posts";

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
        priv_key: &crate::hash::PrivateKey,
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
        bytes.extend(self.timestamp.to_le_bytes());
        bytes
    }

    fn sign(&mut self, priv_key: &crate::hash::PrivateKey) {
        let to_sign = self.sign_bytes();
        self.signature = priv_key.sign(&to_sign);
    }

    pub fn verify(&self) -> bool {
        let to_verify = self.sign_bytes();
        self.source.verify(&to_verify, &self.signature)
    }
}
