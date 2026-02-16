use deadpool_sqlite::Connection;
use rand::seq::{IteratorRandom, SliceRandom};
use surrealdb::{RecordId, Surreal, engine::local::Db};
use tracing::info;

use crate::{errors::DatabaseError, hash::PublicKey};

use super::User;

pub struct UserRepository(Connection);

impl UserRepository {
    pub fn new(conn: Connection) -> UserRepository {
        UserRepository(conn)
    }
}

impl UserRepository {
    pub async fn upsert_user(&self, users: User) -> Result<User, DatabaseError> {
        todo!()
        // // if user.verify() {
        // //     return Err(DatabaseError::InvalidSignature);
        // // }

        // let result: Option<User> = self
        //     .db
        //     .upsert(("users", users.pub_key().to_base64()))
        //     .content(users)
        //     .await?;

        // match result {
        //     Some(user) => {
        //         info!("Created user: {}", user.name());
        //         Ok(user)
        //     }
        //     None => Err(DatabaseError::Unknown),
        // }
    }

    pub async fn update_user(&self, user: User) -> Result<User, DatabaseError> {
        todo!()
        // let result: Option<User> = self
        //     .db
        //     .update(("users", user.pub_key().to_base64()))
        //     .content(user)
        //     .await?;

        // match result {
        //     Some(user) => {
        //         info!("Updated user: {}", user.name());
        //         Ok(user)
        //     }
        //     None => Err(DatabaseError::Unknown),
        // }
    }

    pub async fn get_users_b64(
        &self,
        pub_keys_base64: Vec<String>,
    ) -> Result<Vec<User>, DatabaseError> {
        todo!()
        // let ids: Vec<RecordId> = pub_keys_base64
        //     .iter()
        //     .map(|p| RecordId::from(("users", p)))
        //     .collect();

        // let results: Vec<User> = self
        //     .db
        //     .query("SELECT * FROM $ids")
        //     .bind(("ids", ids))
        //     .await?
        //     .take(0)?;

        // Ok(results)
    }

    pub async fn get_users(&self, pub_keys: Vec<PublicKey>) -> Result<Vec<User>, DatabaseError> {
        todo!()
        // let ids: Vec<RecordId> = pub_keys
        //     .iter()
        //     .map(|p| RecordId::from(("users", p.to_base64())))
        //     .collect();

        // let results: Vec<User> = self
        //     .db
        //     .query("SELECT * FROM $ids")
        //     .bind(("ids", ids))
        //     .await?
        //     .take(0)?;

        // Ok(results)
    }

    pub async fn get_random_user(&self) -> Result<User, DatabaseError> {
        // let results: Vec<User> = self.db.select("users").await.unwrap();
        // let user = results.into_iter().choose(&mut rand::thread_rng());
        // match user {
        //     Some(user) => Ok(user),
        //     None => Err(DatabaseError::Unknown),
        // }
        todo!()
    }

    pub async fn get_all_users(&self) -> Vec<User> {
        // let results: Vec<User> = self.db.select("users").await.unwrap();
        // results
        let results: i64 = self
            .0
            .interact(|conn| async move {
                // let mut stmt = conn.prepare("SELECT * FROM users").unwrap();
                // let mut rows = stmt.query([]).unwrap();
                // let row = rows.next().unwrap().unwrap();
                0i64
            })
            .await
            .unwrap()
            .await;
        todo!()
    }

    pub async fn get_user(&self, pub_key: &PublicKey) -> Option<User> {
        // let results: Option<User> = self
        //     .db
        //     .select(("users", pub_key.to_base64()))
        //     .await
        //     .unwrap();

        // results
        todo!()
    }
}
