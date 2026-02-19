use crate::{
    db::{
        IndexTag, Repositories, TaggedIndex,
        index::{IndexRepository, MangaTag},
    },
    server::{ServerState, handler::AuroraProtocolCommand, protocol::AuroraProtocolResponse},
};

pub struct GetAllIndexes;

impl AuroraProtocolCommand for GetAllIndexes {
    type RequestPayload = GetAllIndexesRequest;
    type ResponsePayload = GetAllIndexesResponse;

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
    ) -> AuroraProtocolResponse<Self::ResponsePayload> {
        match req.tag.as_str() {
            MangaTag::TAG => {
                let indexes = state.repositories.index().await.get_indexes().await;
                AuroraProtocolResponse::ok(GetAllIndexesResponse {
                    indexes: indexes.into_iter().map(TaggedIndex::from).collect(),
                })
            }
            _ => AuroraProtocolResponse::invalid_argument(format!("Invalid tag: {}", req.tag)),
        }
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetAllIndexesRequest {
    pub tag: String,
}

impl GetAllIndexesRequest {
    pub fn new<T: IndexTag>(_tag: T) -> Self {
        Self {
            tag: T::TAG.to_string(),
        }
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetAllIndexesResponse {
    pub indexes: Vec<TaggedIndex>,
}
