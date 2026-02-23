use std::marker::PhantomData;

use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    db::{
        Index, IndexTag, NoTag, Repositories,
        index::{IndexRepository, MangaTag},
        user::I2PAddress,
    },
    errors::{DecodeError, EncodeError},
    helpers::Byteable,
    server::{ServerState, handler::AuroraProtocolCommand, protocol::AuroraProtocolResponse},
};

pub struct GetAllIndexes;

impl AuroraProtocolCommand for GetAllIndexes {
    type RequestPayload = GetAllIndexesRequest;
    type ResponsePayload = GetAllIndexesResponse;
    type ResponseData = Index<NoTag>;

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
        address: &I2PAddress,
    ) -> AuroraProtocolResponse<Self::ResponsePayload, Self::ResponseData> {
        match req.tag.as_str() {
            MangaTag::TAG => {
                let indexes = match state
                    .repositories
                    .index()
                    .await
                    .get_all_indexes::<MangaTag>()
                    .await
                {
                    Ok(indexes) => indexes,
                    Err(_) => {
                        return AuroraProtocolResponse::internal_error(format!("Database error"));
                    }
                };

                // SAFETY: They are all the same type, just different tags
                AuroraProtocolResponse::ok_with_data(GetAllIndexesResponse {}, unsafe {
                    std::mem::transmute(indexes)
                })
            }
            _ => AuroraProtocolResponse::invalid_argument(format!("Invalid tag: {}", req.tag)),
        }
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetAllIndexesRequest {
    tag: String,
}

impl GetAllIndexesRequest {
    pub fn new<T: IndexTag>() -> Self {
        Self {
            tag: T::TAG.to_string(),
        }
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetAllIndexesResponse {}
