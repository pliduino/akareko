// use serde_json::value::Index;
// use tokio::io::{AsyncRead, AsyncWrite};

// use crate::{
//     db::{
//         IndexResponse, IndexTag, Repositories,
//         index::{IndexRepository, NovelTag},
//     },
//     errors::DecodeError,
//     hash::Hash,
//     server::{
//         ServerState,
//         handler::AuroraProtocolCommand,
//         protocol::{AuroraProtocolResponse, byteable::Byteable},
//     },
// };

// pub struct GetContent;

// impl AuroraProtocolCommand for GetContent {
//     type RequestPayload = GetContentRequest;
//     type ResponsePayload = GetContentResponse;

//     async fn process<R: Repositories + 'static>(
//         req: Self::RequestPayload,
//         state: ServerState<R>,
//     ) -> AuroraProtocolResponse<Self> {
//     }
// }

// pub struct GetContentRequest {
//     pub index: Hash,
// }

// impl GetContentRequest {
//     pub fn new(index: Hash) -> Self {
//         Self { index }
//     }
// }

// impl Byteable for GetContentRequest {
//     async fn encode<W: AsyncWrite + Unpin + Send>(&self, writer: &mut W) -> std::io::Result<()> {
//         self.index.encode(writer).await
//     }

//     async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError>
//     where
//         Self: Sized,
//     {
//         let index = Hash::decode(reader).await?;
//         Ok(Self { index })
//     }
// }

// pub struct GetContentResponse {
//     pub indexes: Vec<IndexResponse>,
// }

// impl Byteable for GetContentResponse {
//     async fn encode<W: AsyncWrite + Unpin + Send>(&self, writer: &mut W) -> std::io::Result<()> {
//         self.indexes.encode(writer).await
//     }

//     async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError>
//     where
//         Self: Sized,
//     {
//         let indexes = Vec::<IndexResponse>::decode(reader).await?;
//         Ok(Self { indexes })
//     }
// }
