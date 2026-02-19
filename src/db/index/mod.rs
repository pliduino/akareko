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
pub struct MangaTag;

impl IndexTag for MangaTag {
    const TAG: &'static str = "mangas";
    const CONTENT_TABLE: &'static str = "manga_chapters";
    type Content = MangaChapter;
}

/// Helper for vectors with multiple tags
#[derive(Debug, Clone)]
pub enum TaggedContent {
    Manga(Content<MangaTag>),
}

impl TaggedContent {
    pub fn index_hash(&self) -> &Hash {
        match self {
            TaggedContent::Manga(content) => content.index_hash(),
        }
    }

    pub fn verify(&self) -> bool {
        match self {
            TaggedContent::Manga(content) => content.verify(),
        }
    }
}

impl From<Content<MangaTag>> for TaggedContent {
    fn from(value: Content<MangaTag>) -> Self {
        TaggedContent::Manga(value)
    }
}

impl Byteable for TaggedContent {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        match self {
            TaggedContent::Manga(content) => {
                MangaTag::TAG.to_string().encode(writer).await?;
                content.encode(writer).await
            }
        }
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        let tag = String::decode(reader).await?;
        match tag.as_str() {
            MangaTag::TAG => Ok(TaggedContent::Manga(
                Content::<MangaTag>::decode(reader).await?,
            )),
            _ => Err(DecodeError::InvalidEnumVariant {
                variant_value: tag,
                enum_name: stringify!(TaggedContent),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, byteable_derive::Byteable)]
pub struct MangaChapter {
    pub language: Language,
}

impl MangaChapter {
    pub fn new(language: Language) -> MangaChapter {
        MangaChapter { language }
    }
}

impl ToBytes for MangaChapter {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend((self.language.clone() as u16).to_be_bytes());
        bytes
    }
}
