use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use surrealdb::{Surreal, engine::local::Db, types::RecordId};
use surrealdb_types::SurrealValue;
use tracing::info;

use crate::{
    db::{Content, Index, IndexTag},
    errors::DatabaseError,
    hash::{Hash, PublicKey, Signature},
};

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
        let created: Vec<Index<T>> = self.db.upsert(T::TAG).content(index).await?;

        match created.len() {
            1 => Ok(created.into_iter().next().unwrap()),
            _ => Err(DatabaseError::Unknown),
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

    pub async fn get_all_indexes<T: IndexTag>(&self) -> Result<Vec<Index<T>>, DatabaseError> {
        let results: Vec<Index<T>> = self.db.select(T::TAG).await?;

        Ok(results)
    }

    pub async fn get_indexes<T: IndexTag>(
        &self,
        hashes: &[Hash],
    ) -> Result<Vec<Index<T>>, DatabaseError> {
        let ids: Vec<RecordId> = hashes
            .iter()
            .map(|h| RecordId::new(T::TAG, h.as_base64()))
            .collect();

        let results: Vec<Index<T>> = self
            .db
            .query("SELECT * FROM $ids")
            .bind(("ids", ids))
            .await?
            .take(0)?;

        Ok(results)
    }

    pub async fn get_index<T: IndexTag>(
        &self,
        hash: &Hash,
    ) -> Result<Option<Index<T>>, DatabaseError> {
        let result: Option<Index<T>> = self.db.select((T::TAG, hash.as_base64())).await?;
        Ok(result)
    }

    pub async fn get_contents<T: IndexTag>(&self, index_hash: Hash) -> Vec<Content<T>> {
        let query: String = format!(
            "SELECT * FROM {} WHERE index_hash = $index_hash",
            T::CONTENT_TABLE
        );
        let chapters: Vec<Content<T>> = self
            .db
            .query(query)
            .bind(("index_hash", index_hash))
            .await
            .unwrap()
            .take(0)
            .unwrap();

        chapters
    }
}
