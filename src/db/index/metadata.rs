use serde::{Deserialize, Serialize};
use surrealdb_types::SurrealValue;

use crate::types::Hash;

#[derive(Debug, Default, Clone, SurrealValue, Serialize, Deserialize)]
pub enum IndexStatus {
    Completed,
    Hiatus,
    Cancelled,
    Releasing,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, SurrealValue, Serialize, Deserialize)]
pub struct IndexMetadata {
    hash: Hash, // Primary Key
    pub status: IndexStatus,
}
