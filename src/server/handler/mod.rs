use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    db::Repositories,
    errors::{ClientError, DecodeError, EncodeError},
    helpers::Byteable,
    server::{
        ServerState,
        protocol::{AuroraProtocolRequest, AuroraProtocolResponse, AuroraProtocolVersion},
    },
};

pub mod index;
mod macros;
pub mod users;

/// Marker implemented by the handler macro
pub trait CommandEnum: Byteable {}

/// Marker implemented by the handler macro
pub trait CommandCategoryEnum: Byteable {}

/// Should be implemented by each command, can be skipped by directly implementing [`AuroraProtocolCommandHandler`]
pub(super) trait AuroraProtocolCommand: Sized + AuroraProtocolCommandMetadata {
    type RequestPayload: Byteable;
    type ResponsePayload: Byteable;

    // Used by the client
    async fn request<S: AsyncRead + AsyncWrite + Unpin + Send>(
        payload: Self::RequestPayload,
        stream: &mut S,
    ) -> Result<AuroraProtocolResponse<Self::ResponsePayload>, ClientError> {
        let req = AuroraProtocolRequest::<Self> { payload };
        req.encode(stream).await?;
        let res = AuroraProtocolResponse::decode(stream).await?;
        Ok(res)
    }

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
    ) -> AuroraProtocolResponse<Self::ResponsePayload>;
}

trait AuroraProtocolCommandHandler {
    async fn handle<S: AsyncRead + AsyncWrite + Unpin + Send>(stream: &mut S, state: &ServerState);
}

impl<T: AuroraProtocolCommand> AuroraProtocolCommandHandler for T {
    async fn handle<S: AsyncRead + AsyncWrite + Unpin + Send>(stream: &mut S, state: &ServerState) {
        let req = T::RequestPayload::decode(stream).await.unwrap();
        let res = T::process(req, state).await;
        res.encode(stream).await.unwrap();
    }
}

/// Auto implemented by the handler macro, used to encode requests
pub trait AuroraProtocolCommandMetadata {
    type CommandCategory: CommandCategoryEnum;
    type CommandType: CommandEnum;

    const COMMAND_CATEGORY: Self::CommandCategory;
    const COMMAND: Self::CommandType;
    const VERSION: AuroraProtocolVersion;

    async fn encode_request<W: AsyncWrite + Unpin + Send>(
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        Self::VERSION.encode(writer).await?;
        Self::COMMAND_CATEGORY.encode(writer).await?;
        Self::COMMAND.encode(writer).await
    }
}

pub trait AuroraMiddleware {}

crate::handler!(V1, AuroraProtocolVersion::V1, {
    Users(0) => {
        GetUsers(0) => users::GetUsers,
        Who(1) => users::Who
    },
    Index(1) => {
        GetAllIndexes(0) => index::GetAllIndexes,
        ExchangeContent(1) => index::ExchangeContent,
        GetIndexes(2) => index::GetIndexes
    },
});
