use crate::{
    db::{
        IndexTag, Repositories,
        comments::{Post, Topic},
        index::{IndexRepository, MangaTag},
        user::I2PAddress,
    },
    hash::Hash,
    helpers::Byteable,
    server::{ServerState, handler::AuroraProtocolCommand, protocol::AuroraProtocolResponse},
};

pub struct GetPostsByTopic;

impl AuroraProtocolCommand for GetPostsByTopic {
    type RequestPayload = GetPostsByTopicRequest;
    type ResponsePayload = GetPostsByTopicResponse;
    type ResponseData = ();

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
        address: &I2PAddress,
    ) -> AuroraProtocolResponse<Self::ResponsePayload, Self::ResponseData> {
        todo!();
        let posts = state
            .repositories
            .posts()
            .await
            .get_posts_by_topic(req.topic, 2000, 0)
            .await
            .unwrap()
            .values
            .0;

        AuroraProtocolResponse::ok(GetPostsByTopicResponse { posts })
    }

    async fn request<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send>(
        payload: Self::RequestPayload,
        stream: &mut S,
    ) -> Result<
        AuroraProtocolResponse<Self::ResponsePayload, Self::ResponseData>,
        crate::errors::ClientError,
    > {
        let req = crate::server::protocol::AuroraProtocolRequest::<Self> { payload };
        req.encode(stream).await?;
        let res = AuroraProtocolResponse::decode(stream).await?;
        Ok(res)
    }
}

#[derive(byteable_derive::Byteable)]
pub struct GetPostsByTopicRequest {
    pub topic: Topic,
}

#[derive(byteable_derive::Byteable)]
pub struct GetPostsByTopicResponse {
    pub posts: Vec<Post>,
}
