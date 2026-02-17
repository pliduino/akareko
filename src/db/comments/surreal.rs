use std::collections::HashSet;

use const_format::formatcp;
use serde::Deserialize;
use surrealdb::{Surreal, engine::local::Db};
use tracing::info;

use crate::{
    db::{
        PaginateResponse,
        comments::{Post, Topic},
        user::User,
    },
    errors::DatabaseError,
    hash::{Hash, PublicKey, Signature},
};

pub struct PostRepository<'a> {
    db: &'a Surreal<Db>,
}

impl<'a> PostRepository<'a> {
    pub fn new(db: &'a Surreal<Db>) -> PostRepository<'a> {
        PostRepository { db }
    }
}

impl<'a> PostRepository<'a> {
    pub async fn add_comment(&self, post: Post) -> Result<Post, DatabaseError> {
        let result: Option<Post> = self
            .db
            .create((Post::TABLE_NAME, post.signature.as_base64()))
            .content(post)
            .await?;

        match result {
            Some(post) => {
                info!("Created post: {}", post.signature.as_base64());
                Ok(post)
            }
            None => Err(DatabaseError::Unknown),
        }
    }

    pub async fn get_posts_by_topic(
        &self,
        topic: Topic,
        take: usize,
        skip: usize,
    ) -> Result<PaginateResponse<(Vec<Post>, HashSet<User>)>, DatabaseError> {
        const QUERY: &'static str = formatcp!(
            "
            LET $rows = (
                SELECT *
                FROM {0}
                WHERE topic = $topic
                ORDER BY timestamp ASC
                LIMIT $take
                START $skip
            );

            LET $sources = $rows.map(|$r| $r.source);

            RETURN $sources;

            {{
                total: count(
                    SELECT *
                    FROM {0}
                    WHERE topic = $topic
                ),
                data: $rows,
                users: (
                    SELECT *
                    FROM $sources
                )
            }}
            ",
            Post::TABLE_NAME
        );

        #[derive(Deserialize)]
        struct Response {
            total: usize,
            data: Vec<Post>,
            users: HashSet<User>,
        }

        let result: Option<Response> = self
            .db
            .query(QUERY)
            .bind(("topic", topic))
            .bind(("take", take))
            .bind(("skip", skip))
            .await?
            .take(3)?;

        match result {
            Some(r) => Ok(PaginateResponse {
                values: (r.data, r.users),
                total: r.total,
            }),
            None => Err(DatabaseError::Unknown),
        }
    }
}
