use crate::{
    db::{
        IndexTag, Repositories, TaggedIndex,
        index::{IndexRepository, NovelTag},
    },
    hash::Hash,
    server::{ServerState, handler::AuroraProtocolCommand, protocol::AuroraProtocolResponse},
};

pub struct GetIndexes;

impl AuroraProtocolCommand for GetIndexes {
    type RequestPayload = GetIndexesRequest;
    type ResponsePayload = GetIndexesResponse;

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
    ) -> AuroraProtocolResponse<Self::ResponsePayload> {
        let mut handles = Vec::with_capacity(req.indexes.len());
        for (s, hash) in req.indexes {
            let repo = state.repositories.clone();
            let handle = tokio::spawn(async move {
                match s.as_str() {
                    NovelTag::TAG => {
                        let novel_repo = repo.index();
                        match novel_repo.get_index(&hash).await {
                            Ok(index) => match index {
                                Some(i) => Some(i.into()),
                                None => None,
                            },
                            Err(_) => return None,
                        }
                    }
                    _ => None,
                }
            });
            handles.push(handle);
        }

        let mut indexes = Vec::with_capacity(handles.len());
        for handle in handles {
            match handle.await {
                Ok(Some(index)) => indexes.push(index),
                _ => {}
            }
        }

        AuroraProtocolResponse::ok(GetIndexesResponse { indexes })
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetIndexesRequest {
    pub indexes: Vec<(String, Hash)>,
}

#[derive(byteable_derive::Byteable)]
pub struct GetIndexesResponse {
    pub indexes: Vec<TaggedIndex>,
}
