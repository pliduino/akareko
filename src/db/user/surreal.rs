use rand::seq::{IteratorRandom, SliceRandom};
use surrealdb::{RecordId, Surreal, engine::local::Db};
use tracing::info;

use crate::{errors::DatabaseError, hash::PublicKey};

use super::User;

pub struct UserRepository<'a> {
    db: &'a Surreal<Db>,
}

impl<'a> UserRepository<'a> {
    pub fn new(db: &'a Surreal<Db>) -> UserRepository<'a> {
        UserRepository { db }
    }
}

impl<'a> UserRepository<'a> {
    pub async fn upsert_user(&self, user: User) -> Result<User, DatabaseError> {
        let result: Option<User> = self
            .db
            .upsert((User::TABLE_NAME, user.pub_key().to_base64()))
            .content(user)
            .await?;

        match result {
            Some(user) => {
                info!("Created user: {}", user.name());
                Ok(user)
            }
            None => Err(DatabaseError::Unknown),
        }
    }

    pub async fn get_users_b64(
        &self,
        pub_keys_base64: Vec<String>,
    ) -> Result<Vec<User>, DatabaseError> {
        let ids: Vec<RecordId> = pub_keys_base64
            .iter()
            .map(|p| RecordId::from(("users", p)))
            .collect();

        let results: Vec<User> = self
            .db
            .query("SELECT * FROM $ids")
            .bind(("ids", ids))
            .await?
            .take(0)?;

        Ok(results)
    }

    pub async fn get_users(&self, pub_keys: Vec<PublicKey>) -> Result<Vec<User>, DatabaseError> {
        let ids: Vec<RecordId> = pub_keys
            .iter()
            .map(|p| RecordId::from(("users", p.to_base64())))
            .collect();

        let results: Vec<User> = self
            .db
            .query("SELECT * FROM $ids")
            .bind(("ids", ids))
            .await?
            .take(0)?;

        Ok(results)
    }

    pub async fn get_random_user(&self) -> Result<User, DatabaseError> {
        let results: Vec<User> = self.db.select("users").await.unwrap();
        let user = results.into_iter().choose(&mut rand::thread_rng());
        user.ok_or(DatabaseError::Unknown)
    }

    pub async fn get_all_users(&self) -> Vec<User> {
        let results: Vec<User> = self.db.select("users").await.unwrap();
        results
    }

    pub async fn get_user(&self, pub_key: &PublicKey) -> Option<User> {
        let results: Option<User> = self
            .db
            .select(("users", pub_key.to_base64()))
            .await
            .unwrap();

        results
    }
}
