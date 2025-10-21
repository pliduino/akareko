use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    errors::{ClientError, DecodeError, EncodeError},
    helpers::Byteable,
    server::handler::AuroraProtocolCommand,
};

pub mod byteable;

#[repr(u8)]
#[derive(Debug, Clone, byteable_derive::Byteable)]
pub enum AuroraProtocolVersion {
    V1 = 1,
}

#[derive(Debug)]
pub(super) struct AuroraProtocolRequest<C: AuroraProtocolCommand> {
    pub payload: C::RequestPayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuroraStatus {
    Ok,
    NotFound(String),
    InvalidArgument(String),
    InternalError(String),
}

impl AuroraStatus {
    const OK_CODE: u16 = 200;
    const INTERNAL_ERROR_CODE: u16 = 500;
    const INVALID_ARGUMENT_CODE: u16 = 400;
    const NOT_FOUND_CODE: u16 = 404;

    pub fn is_ok(&self) -> bool {
        matches!(self, AuroraStatus::Ok)
    }

    pub fn code(&self) -> u16 {
        match self {
            AuroraStatus::Ok => Self::OK_CODE,
            AuroraStatus::InvalidArgument(_) => Self::INVALID_ARGUMENT_CODE,
            AuroraStatus::NotFound(_) => Self::NOT_FOUND_CODE,
            AuroraStatus::InternalError(_) => Self::INTERNAL_ERROR_CODE,
        }
    }
}

impl Byteable for AuroraStatus {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        writer.write_u16(self.code()).await?;

        match self {
            AuroraStatus::Ok => (),
            AuroraStatus::InvalidArgument(message) => {
                message.encode(writer).await?;
            }
            AuroraStatus::NotFound(message) => {
                message.encode(writer).await?;
            }
            AuroraStatus::InternalError(message) => {
                message.encode(writer).await?;
            }
        }

        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        let code = reader.read_u16().await?;

        let status = match code {
            Self::OK_CODE => AuroraStatus::Ok,
            Self::INVALID_ARGUMENT_CODE => {
                let message = String::decode(reader).await?;
                AuroraStatus::InvalidArgument(message)
            }
            Self::NOT_FOUND_CODE => {
                let message = String::decode(reader).await?;
                AuroraStatus::NotFound(message)
            }
            Self::INTERNAL_ERROR_CODE => {
                let message = String::decode(reader).await?;
                AuroraStatus::InternalError(message)
            }
            _ => {
                return Err(DecodeError::InvalidEnumVariant {
                    enum_name: "AuroraStatus",
                    variant_value: code.to_string(),
                });
            }
        };

        Ok(status)
    }
}

#[derive(Debug)]
pub(super) struct AuroraProtocolResponse<P: Byteable> {
    status: AuroraStatus,
    payload: Option<P>, // None if status is an error
}

impl<P: Byteable> AuroraProtocolResponse<P> {
    pub fn ok(payload: P) -> Self {
        Self {
            status: AuroraStatus::Ok,
            payload: Some(payload),
        }
    }

    pub fn not_found(message: String) -> Self {
        Self {
            status: AuroraStatus::NotFound(message),
            payload: None,
        }
    }

    pub fn invalid_argument(message: String) -> Self {
        Self {
            status: AuroraStatus::InvalidArgument(message),
            payload: None,
        }
    }

    pub fn internal_error(message: String) -> Self {
        Self {
            status: AuroraStatus::InternalError(message),
            payload: None,
        }
    }

    pub fn status(&self) -> &AuroraStatus {
        &self.status
    }

    pub fn payload(self) -> Option<P> {
        self.payload
    }

    pub fn payload_if_ok(self) -> Result<P, ClientError> {
        if !self.status().is_ok() {
            return Err(ClientError::UnexpectedResponseCode {
                status: self.status,
            });
        }

        let Some(contents) = self.payload() else {
            return Err(ClientError::MissingPayload);
        };

        return Ok(contents);
    }
}

impl<C: AuroraProtocolCommand> AuroraProtocolRequest<C> {
    pub async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        C::encode_request(writer).await?;
        self.payload.encode(writer).await
    }
}

impl<P: Byteable> Byteable for AuroraProtocolResponse<P> {
    async fn encode<W: AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        self.status.encode(writer).await?;
        if let Some(payload) = &self.payload {
            payload.encode(writer).await?;
        }

        Ok(())
    }

    async fn decode<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self, DecodeError> {
        let status = AuroraStatus::decode(reader).await?;

        if !status.is_ok() {
            return Ok(AuroraProtocolResponse {
                status,
                payload: None,
            });
        }

        let response = P::decode(reader).await?;
        Ok(AuroraProtocolResponse {
            status,
            payload: Some(response),
        })
    }
}
