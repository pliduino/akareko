use fastbloom::BloomFilter;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use skerry::skerry;
use strum_macros::EnumCount;
use surrealdb::{Surreal, engine::local::Db, method::Transaction};
use surrealdb_types::{SurrealValue, Value};

use crate::{
    db::{
        BLOOM_FILTER_FALSE_POSITIVE_RATE, Timestamp,
        index::tags::{IndexTag, MangaTag},
    },
    errors::DatabaseError,
    types::Topic,
};

#[derive(SurrealValue, Debug, Clone)]
pub struct Event {
    pub timestamp: Timestamp,
    pub event_type: EventType,
    pub topic: Topic,
}

pub async fn insert_event(events: Vec<Event>, db: &Transaction<Db>) -> Result<(), DatabaseError> {
    let _: Vec<Value> = db.insert("events").content(events).await?;
    Ok(())
}

pub async fn remove_event(topic: Topic, db: &Transaction<Db>) -> Result<(), DatabaseError> {
    const QUERY_STR: &str = "DELETE FROM events WHERE topic = $topic;";

    db.query(QUERY_STR).bind(("topic", topic)).await?;

    Ok(())
}

pub async fn get_paginated_events(
    page: usize,
    per_page: usize,
    db: &Surreal<Db>,
) -> Result<(Vec<Event>, usize), DatabaseError> {
    const QUERY: &str = "
        LET $rows = (
            SELECT *
            FROM events
            ORDER BY timestamp DESC
            LIMIT $take
            START $skip
        );

        {{
            total: count(
                SELECT *
                FROM events
            ),
            data: $rows
        }}
        ";

    #[derive(SurrealValue)]
    struct Response {
        total: usize,
        data: Vec<Event>,
    }

    let events: Vec<Response> = db
        .query(QUERY)
        .bind(("take", per_page))
        .bind(("skip", (page - 1) * per_page))
        .await?
        .take(1)?;

    if let Some(response) = events.into_iter().next() {
        return Ok((response.data, (response.total / per_page) + 1));
    }

    Err(DatabaseError::Unknown)
}

#[skerry]
pub async fn filter_events(
    timestamp: Timestamp,
    filter: Option<BloomFilter>,
    db: &Surreal<Db>,
) -> Result<Vec<(EventType, Vec<Topic>)>, e![Surreal]> {
    const QUERY: &'static str = "
                SELECT event_type, array::group(topic) AS topics FROM events WHERE timestamp >= $timestamp GROUP BY event_type;
            ";

    #[derive(SurrealValue)]
    struct Grouped {
        event_type: EventType,
        topics: Vec<Topic>,
    }

    let events: Vec<Grouped> = db
        .query(QUERY)
        .bind(("timestamp", timestamp))
        .await?
        .take(0)?;

    let mut response = vec![];

    for event in events {
        if let Some(filter) = &filter {
            response.push((
                event.event_type,
                event
                    .topics
                    .into_iter()
                    .filter(|e| {
                        println!("Checking {:?} {}", &e, !filter.contains(e));
                        !filter.contains(e)
                    })
                    .collect(),
            ));
        } else {
            response.push((event.event_type, event.topics));
        }
    }

    Ok(response)
}

pub async fn make_event_filter(
    timestamp: Timestamp,
    db: &Surreal<Db>,
) -> Result<BloomFilter, DatabaseError> {
    const QUERY: &'static str = "
        SELECT topic FROM events WHERE timestamp >= $timestamp ;
    ";

    #[derive(SurrealValue, Hash, Debug)]
    struct TopicWrapper {
        topic: Topic,
    }

    let topics: Vec<TopicWrapper> = db
        .query(QUERY)
        .bind(("timestamp", timestamp))
        .await?
        .take(0)?;

    let mut filter =
        BloomFilter::with_false_pos(BLOOM_FILTER_FALSE_POSITIVE_RATE).expected_items(topics.len());
    filter.insert_all(&topics);
    Ok(filter)
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    IntoPrimitive,
    TryFromPrimitive,
    SurrealValue,
    EnumCount,
    Serialize,
    Deserialize,
)]
#[repr(u8)]
pub enum EventType {
    Invalid = 0,
    User = 1,
    Manga = 2,
    MangaContent = 3,
    Post = 4,
}

impl EventType {
    pub fn as_str(&self) -> &str {
        match self {
            EventType::Invalid => "",
            EventType::User => "user",
            EventType::Manga => MangaTag::TAG,
            EventType::MangaContent => MangaTag::CONTENT_TABLE,
            EventType::Post => "post",
        }
    }
}

impl std::hash::Hash for Event {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.topic.hash(state);
    }
}

#[cfg(test)]
mod tests {

    use uuid::Uuid;

    use crate::{
        db::{
            Repositories,
            index::{Index, IndexLinks},
        },
        types::PrivateKey,
    };

    use super::*;

    #[tokio::test]
    async fn test_filter_events() {
        let repo = Repositories::in_memory().await;
        let index = Index::<MangaTag>::new_signed(
            "test".to_string(),
            0,
            IndexLinks {
                myanimelist: None,
                mangadex: Some(Uuid::parse_str("410d499a-f438-4a56-9ad4-eb90a4de5b39").unwrap()),
            },
            &PrivateKey::new(),
        );

        repo.index().add_index(index.clone()).await.unwrap();

        let filter = make_event_filter(Timestamp::new(0), &repo.db)
            .await
            .unwrap();

        let topic = Topic::from_index(&index);

        assert!(filter.contains(&topic));
    }
}
