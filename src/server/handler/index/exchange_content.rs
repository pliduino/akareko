use crate::{
    db::{Repositories, index::TaggedContent},
    hash::Hash,
    server::{ServerState, handler::AuroraProtocolCommand, protocol::AuroraProtocolResponse},
};

pub struct ExchangeContent;

impl AuroraProtocolCommand for ExchangeContent {
    type RequestPayload = ExchangeContentRequest;
    type ResponsePayload = ExchangeContentResponse;

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
    ) -> AuroraProtocolResponse<Self::ResponsePayload> {
        let Ok(contents) = state.repositories.get_random_contents(req.count).await else {
            return AuroraProtocolResponse::internal_error(
                "Failed to get random indexes".to_string(),
            );
        };

        AuroraProtocolResponse::ok(ExchangeContentResponse { contents })
    }
}

#[derive(byteable_derive::Byteable)]
pub struct ExchangeContentRequest {
    pub count: u16,
}

#[derive(byteable_derive::Byteable)]
pub struct ExchangeContentResponse {
    pub contents: Vec<TaggedContent>,
}
