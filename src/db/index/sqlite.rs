use deadpool_sqlite::Connection;

use crate::db::{Content, Index, IndexTag};
use crate::errors::DatabaseError;
use crate::hash::Hash;

pub struct IndexRepository(Connection);

impl IndexRepository {
    pub fn new(conn: Connection) -> IndexRepository {
        IndexRepository(conn)
    }
}

impl IndexRepository {
    pub async fn add_index<T: IndexTag>(&self, index: Index<T>) -> Result<Index<T>, DatabaseError> {
        todo!()
        // let index: IndexSurrealDb = index.into();

        // let created: Result<Option<IndexSurrealDb>, surrealdb::Error> =
        //     self.db.upsert(index.id.clone()).content(index).await;

        // match created {
        //     Ok(i) => match i {
        //         Some(i) => {
        //             // info!("Created {}: {}", i.tag(), i.title());
        //             Ok(i.into())
        //         }
        //         None => Err(DatabaseError::Unknown),
        //     },
        //     Err(_) => Err(DatabaseError::Unknown),
        // }
    }

    pub async fn add_content<T: IndexTag + 'static>(
        &self,
        content: Content<T>,
    ) -> Result<Content<T>, DatabaseError> {
        todo!()
        // let created: Result<Option<Content<T>>, surrealdb::Error> = self
        //     .db
        //     .upsert((T::CONTENT_TABLE, content.signature.as_base64()))
        //     .content(content)
        //     .await;

        // match created {
        //     Ok(n) => match n {
        //         Some(n) => Ok(n),
        //         None => Err(DatabaseError::Unknown),
        //     },
        //     Err(e) => {
        //         info!("Error: {}", e);
        //         Err(DatabaseError::Unknown)
        //     }
        // }
    }

    pub async fn get_indexes<T: IndexTag>(&self) -> Vec<Index<T>> {
        // let results: Vec<IndexSurrealDb> = self.db.select(T::TAG).await.unwrap();
        // results.into_iter().map(|i| i.into()).collect()
        todo!()
    }

    pub async fn get_index<T: IndexTag>(
        &self,
        hash: &Hash,
    ) -> Result<Option<Index<T>>, DatabaseError> {
        // let result: Option<IndexSurrealDb> = self.db.select((T::TAG, hash.as_base64())).await?;
        // Ok(result.map(|i| i.into()))
        todo!()
    }

    pub async fn get_contents<T: IndexTag>(&self, index_hash: Hash) -> Vec<Content<T>> {
        todo!()
        // let chapters: Vec<Content<T>> = self
        //     .db
        //     .query(format!(
        //         "SELECT * FROM {} WHERE index_hash = $index_hash",
        //         T::CONTENT_TABLE
        //     ))
        //     .bind(("index_hash", index_hash))
        //     .await
        //     .unwrap()
        //     .take(0)
        //     .unwrap();

        // chapters
    }
}
