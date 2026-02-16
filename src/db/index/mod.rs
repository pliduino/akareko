use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    db::{Content, Index, IndexTag, ToBytes},
    errors::{DatabaseError, DecodeError, EncodeError},
    hash::Hash,
    helpers::{Byteable, Language},
};

#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(feature = "sqlite")]
pub use sqlite::IndexRepository;

#[cfg(feature = "surrealdb")]
mod surreal;
#[cfg(feature = "surrealdb")]
pub use surreal::IndexRepository;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelTag;

impl IndexTag for NovelTag {
    const TAG: &'static str = "novels";
    const CONTENT_TABLE: &'static str = "novel_chapters";
    type Content = NovelChapter;
}

/// Helper for vectors with multiple tags
#[derive(Debug, Clone)]
pub enum TaggedContent {
    Novel(Content<NovelTag>),
}

impl TaggedContent {
    pub fn index_hash(&self) -> &Hash {
        match self {
            TaggedContent::Novel(content) => content.index_hash(),
        }
    }

    pub fn verify(&self) -> bool {
        match self {
            TaggedContent::Novel(content) => content.verify(),
        }
    }
}

impl From<Content<NovelTag>> for TaggedContent {
    fn from(value: Content<NovelTag>) -> Self {
        TaggedContent::Novel(value)
    }
}

impl Byteable for TaggedContent {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        match self {
            TaggedContent::Novel(content) => {
                NovelTag::TAG.to_string().encode(writer).await?;
                content.encode(writer).await
            }
        }
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        let tag = String::decode(reader).await?;
        match tag.as_str() {
            NovelTag::TAG => Ok(TaggedContent::Novel(
                Content::<NovelTag>::decode(reader).await?,
            )),
            _ => Err(DecodeError::InvalidEnumVariant {
                variant_value: tag,
                enum_name: stringify!(TaggedContent),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, byteable_derive::Byteable)]
pub struct NovelChapter {
    pub language: Language,
}

impl NovelChapter {
    pub fn new(language: Language) -> NovelChapter {
        NovelChapter { language }
    }
}

impl ToBytes for NovelChapter {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend((self.language.clone() as u16).to_be_bytes());
        bytes
    }
}
