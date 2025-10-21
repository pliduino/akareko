use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use surrealdb::{RecordId, Surreal, engine::local::Db};
use tracing::info;

use crate::{
    db::{Content, Index, IndexTag, Repositories},
    errors::DatabaseError,
    hash::{Hash, PublicKey, Signature},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexSurrealDb {
    id: surrealdb::RecordId,
    title: String,
    release_date: i32,
    source: PublicKey,
    signature: Signature,
}

impl<T: IndexTag> From<Index<T>> for IndexSurrealDb {
    fn from(index: Index<T>) -> Self {
        let Index {
            hash,
            title,
            release_date,
            source,
            signature,
            ..
        } = index;

        Self {
            id: RecordId::from_table_key(T::TAG.to_string(), hash.as_base64()),
            title,
            release_date,
            source,
            signature,
        }
    }
}

impl<T: IndexTag> Into<Index<T>> for IndexSurrealDb {
    fn into(self) -> Index<T> {
        let IndexSurrealDb {
            id,
            title,
            release_date,
            source,
            signature,
        } = self;

        let key = &id.key().to_string();
        let trimmed = key.trim_start_matches("⟨").trim_end_matches("⟩");

        Index {
            hash: Hash::from_base64(trimmed).unwrap(),
            title,
            release_date,
            source,
            signature,
            _phantom: PhantomData,
        }
    }
}

pub struct IndexRepository<'a> {
    db: &'a Surreal<Db>,
}

impl<'a> IndexRepository<'a> {
    pub fn new(db: &'a Surreal<Db>) -> IndexRepository<'a> {
        IndexRepository { db }
    }
}

impl<'a> IndexRepository<'a> {
    pub async fn add_index<T: IndexTag>(&self, index: Index<T>) -> Result<Index<T>, DatabaseError> {
        let index: IndexSurrealDb = index.into();

        let created: Result<Option<IndexSurrealDb>, surrealdb::Error> =
            self.db.upsert(index.id.clone()).content(index).await;

        match created {
            Ok(i) => match i {
                Some(i) => {
                    // info!("Created {}: {}", i.tag(), i.title());
                    Ok(i.into())
                }
                None => Err(DatabaseError::Unknown),
            },
            Err(_) => Err(DatabaseError::Unknown),
        }
    }

    pub async fn add_content<T: IndexTag + 'static>(
        &self,
        content: Content<T>,
    ) -> Result<Content<T>, DatabaseError> {
        let created: Result<Option<Content<T>>, surrealdb::Error> = self
            .db
            .upsert((T::CONTENT_TABLE, content.signature.as_base64()))
            .content(content)
            .await;

        match created {
            Ok(n) => match n {
                Some(n) => Ok(n),
                None => Err(DatabaseError::Unknown),
            },
            Err(e) => {
                info!("Error: {}", e);
                Err(DatabaseError::Unknown)
            }
        }
    }

    pub async fn get_indexes<T: IndexTag>(&self) -> Vec<Index<T>> {
        let results: Vec<IndexSurrealDb> = self.db.select(T::TAG).await.unwrap();
        results.into_iter().map(|i| i.into()).collect()
    }

    pub async fn get_index<T: IndexTag>(
        &self,
        hash: &Hash,
    ) -> Result<Option<Index<T>>, DatabaseError> {
        let result: Option<IndexSurrealDb> = self.db.select((T::TAG, hash.as_base64())).await?;
        Ok(result.map(|i| i.into()))
    }

    pub async fn get_contents<T: IndexTag>(&self, index_hash: Hash) -> Vec<Content<T>> {
        let chapters: Vec<Content<T>> = self
            .db
            .query(format!(
                "SELECT * FROM {} WHERE index_hash = $index_hash",
                T::CONTENT_TABLE
            ))
            .bind(("index_hash", index_hash))
            .await
            .unwrap()
            .take(0)
            .unwrap();

        chapters
    }
}
