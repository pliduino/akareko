use std::{fmt::Debug, marker::PhantomData};

use futures::SinkExt;
use rclite::Arc;
use serde::{Deserialize, Deserializer, Serialize};
use surrealdb::{
    RecordId, Surreal,
    engine::local::{Db, SurrealKv},
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    sync::RwLock,
};
use tracing::info;

use crate::{
    config::AuroraConfig,
    db::{
        index::{IndexRepository, NovelTag, TaggedContent},
        user::{User, UserRepository},
    },
    errors::{DatabaseError, DecodeError, EncodeError},
    hash::{Hash, PrivateKey, PublicKey, Signature},
    helpers::{Byteable, SanitizedString, now_timestamp},
};

pub mod index;
pub mod user;

pub type Timestamp = u64;

pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

impl ToBytes for () {
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Magnet(pub String);

impl Byteable for Magnet {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        self.0.encode(writer).await
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(Magnet(String::decode(reader).await?))
    }
}

#[derive(Debug, Clone)]
pub struct Repositories {
    pub db: Surreal<Db>,
    config: Arc<RwLock<AuroraConfig>>,
}

impl Repositories {
    pub async fn initialize(config: Arc<RwLock<AuroraConfig>>) -> Self {
        info!("Initializing SurrealDB");
        let db = Surreal::new::<SurrealKv>("database").await.unwrap();
        db.use_ns("aurora").use_db("main").await.unwrap();
        let repositories = Repositories { db, config };
        info!("Initialized SurrealDB");

        {
            let config = repositories.config.read().await;

            let user_repository = repositories.user();
            match user_repository.get_user(&config.public_key()).await {
                Some(_) => {}
                None => {
                    let user = User::new_signed(
                        "Anon".to_string(),
                        now_timestamp(),
                        &config.private_key(),
                        Some(config.eepsite_address().clone()),
                    );
                    user_repository.upsert_user(user).await.unwrap();
                }
            }
        }

        repositories
    }

    pub async fn get_random_contents(
        &self,
        count: u16,
    ) -> Result<Vec<TaggedContent>, DatabaseError> {
        let mut tagged_contents = Vec::with_capacity(count as usize);

        let novel_tag = count;

        let novels: Vec<Content<NovelTag>> = self
            .db
            .query(format!(
                "SELECT * FROM {} ORDER BY rand() LIMIT $count",
                NovelTag::CONTENT_TABLE
            ))
            .bind(("count", novel_tag))
            .await?
            .take(0)?;

        tagged_contents.extend(novels.into_iter().map(TaggedContent::from));

        Ok(tagged_contents)
    }

    pub fn user(&self) -> UserRepository {
        UserRepository::new(&self.db)
    }

    pub fn index(&self) -> IndexRepository {
        IndexRepository::new(&self.db)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, byteable_derive::Byteable)]
#[repr(u8)]
pub enum ContentReadType {
    SequenceFolder,
    SingleFile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEntry<T: IndexTag> {
    pub title: String,
    pub enumeration: f32,
    pub path: String,
    pub ty: ContentReadType,
    pub content: T::Content,
}

impl<T: IndexTag> ToBytes for ContentEntry<T> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = self.title.as_bytes().to_vec();
        bytes.extend(self.enumeration.to_be_bytes());
        bytes.extend(self.path.as_bytes());
        bytes.push(self.ty.clone() as u8);
        bytes.extend(self.content.to_bytes());
        bytes
    }
}

impl<T: IndexTag> Byteable for ContentEntry<T> {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        self.title.encode(writer).await?;
        self.enumeration.encode(writer).await?;
        self.path.encode(writer).await?;
        self.ty.encode(writer).await?;
        self.content.encode(writer).await?;
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError>
    where
        Self: Sized,
    {
        Ok(ContentEntry {
            title: String::decode(reader).await?,
            enumeration: f32::decode(reader).await?,
            path: String::decode(reader).await?,
            ty: ContentReadType::decode(reader).await?,
            content: T::Content::decode(reader).await?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T::Content: Serialize",
    deserialize = "T::Content: Deserialize<'de>"
))]
pub struct Content<T: IndexTag> {
    #[serde(
        rename = "id",
        skip_serializing,
        deserialize_with = "deserialize_signature_id"
    )]
    signature: Signature,
    source: PublicKey,

    // Signed Fields
    index_hash: Hash,
    pub timestamp: Timestamp,
    pub magnet_link: Magnet,
    entries: Vec<ContentEntry<T>>,
}

fn deserialize_signature_id<'de, D>(deserializer: D) -> Result<Signature, D::Error>
where
    D: Deserializer<'de>,
{
    let id = RecordId::deserialize(deserializer)?;
    let key = id.key().to_string();
    let trimmed = key.trim_start_matches("⟨").trim_end_matches("⟩");

    Ok(Signature::from_base64(&trimmed).unwrap())
}

impl<T: IndexTag> Content<T> {
    pub fn new(
        signature: Signature,
        source: PublicKey,
        index_hash: Hash,
        timestamp: Timestamp,
        magnet_link: Magnet,
        entries: Vec<ContentEntry<T>>,
    ) -> Self {
        Self {
            signature,
            source,
            index_hash,
            timestamp,
            magnet_link,
            entries,
        }
    }

    pub fn id_bytes(
        index_hash: &Hash,
        timestamp: &Timestamp,
        magnet_link: &Magnet,
        entries: &Vec<ContentEntry<T>>,
    ) -> Vec<u8> {
        let mut bytes: Vec<u8> = index_hash.inner().to_vec().to_vec();
        bytes.extend(timestamp.to_be_bytes());
        bytes.extend(magnet_link.0.as_bytes());
        for entry in entries {
            bytes.extend(entry.to_bytes());
        }
        bytes
    }

    pub fn new_signed(
        source: PublicKey,
        index_hash: Hash,
        timestamp: Timestamp,
        magnet_link: Magnet,
        entries: Vec<ContentEntry<T>>,
        priv_key: &PrivateKey,
    ) -> Self {
        let to_sign = Self::id_bytes(&index_hash, &timestamp, &magnet_link, &entries);
        let signature = priv_key.sign(&to_sign);

        Self::new(
            signature,
            source,
            index_hash,
            timestamp,
            magnet_link,
            entries,
        )
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    pub fn entries(&self) -> &Vec<ContentEntry<T>> {
        &self.entries
    }

    pub fn index_hash(&self) -> &Hash {
        &self.index_hash
    }
}

impl<T: IndexTag> Byteable for Content<T> {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        self.signature.encode(writer).await?;
        self.source.encode(writer).await?;
        self.index_hash.encode(writer).await?;
        self.timestamp.encode(writer).await?;
        self.magnet_link.encode(writer).await?;
        self.entries.encode(writer).await?;
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(Content {
            signature: Signature::decode(reader).await?,
            source: PublicKey::decode(reader).await?,
            index_hash: Hash::decode(reader).await?,
            timestamp: Timestamp::decode(reader).await?,
            magnet_link: Magnet::decode(reader).await?,
            entries: Vec::decode(reader).await?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Index<T: IndexTag> {
    hash: Hash, // Primary Key
    title: String,
    release_date: i32,
    source: PublicKey,
    signature: Signature,
    _phantom: PhantomData<T>,
}

pub trait IndexTag: Send + Clone + Debug {
    const TAG: &'static str; // Acts like table name
    const CONTENT_TABLE: &'static str;
    type Content: Send + Serialize + for<'de> Deserialize<'de> + Clone + Debug + ToBytes + Byteable;
}

impl<T: IndexTag> Index<T> {
    pub fn new(title: String, release_date: i32, source: PublicKey, signature: Signature) -> Self {
        let hash = Hash::digest(&Self::id_bytes(&title, &release_date));

        Self {
            hash,
            title,
            release_date,
            source,
            signature,
            _phantom: PhantomData,
        }
    }

    pub fn transmute<T2: IndexTag>(self) -> Index<T2> {
        Index {
            hash: self.hash,
            title: self.title,
            release_date: self.release_date,
            source: self.source,
            signature: self.signature,
            _phantom: PhantomData,
        }
    }

    fn id_bytes(title: &String, release_date: &i32) -> Vec<u8> {
        let sanitized_title = SanitizedString::new(&title);

        let mut bytes = T::TAG.as_bytes().to_vec();
        bytes.extend(sanitized_title.as_bytes());
        bytes.extend(release_date.to_le_bytes());
        bytes
    }

    pub fn new_signed(title: String, release_date: i32, priv_key: &PrivateKey) -> Self {
        let mut index = Self::new(
            title,
            release_date,
            priv_key.public_key(),
            Signature::empty(),
        );

        index.sign_index(priv_key);

        index
    }

    fn sign_index(&mut self, priv_key: &PrivateKey) {
        let to_sign = Self::id_bytes(&self.title, &self.release_date);
        self.signature = priv_key.sign(&to_sign);
    }

    pub fn hash(&self) -> &Hash {
        &self.hash
    }

    pub fn title(&self) -> &String {
        &self.title
    }

    pub fn release_date(&self) -> i32 {
        self.release_date
    }

    pub fn source(&self) -> &PublicKey {
        &self.source
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

impl<T: IndexTag> Byteable for Index<T> {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        self.hash.encode(writer).await?;
        self.title.encode(writer).await?;
        self.release_date.encode(writer).await?;
        self.source.encode(writer).await?;
        self.signature.encode(writer).await?;
        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        Ok(Index {
            hash: Hash::decode(reader).await?,
            title: String::decode(reader).await?,
            release_date: i32::decode(reader).await?,
            source: PublicKey::decode(reader).await?,
            signature: Signature::decode(reader).await?,
            _phantom: PhantomData,
        })
    }
}

pub enum TaggedIndex {
    Novel(Index<NovelTag>),
}

impl Byteable for TaggedIndex {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        match self {
            TaggedIndex::Novel(index) => {
                NovelTag::TAG.to_string().encode(writer).await?;
                index.encode(writer).await?;
            }
        }

        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        let tag = String::decode(reader).await?;
        match tag.as_str() {
            NovelTag::TAG => Ok(TaggedIndex::Novel(Index::decode(reader).await?)),
            _ => Err(DecodeError::InvalidEnumVariant {
                variant_value: tag,
                enum_name: stringify!(TaggedIndex),
            }),
        }
    }
}

impl From<Index<NovelTag>> for TaggedIndex {
    fn from(index: Index<NovelTag>) -> Self {
        TaggedIndex::Novel(index)
    }
}
