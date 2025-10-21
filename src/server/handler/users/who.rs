use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    db::{
        Repositories,
        user::{I2PAddress, UserRepository},
    },
    errors::{DecodeError, EncodeError},
    hash::{Signable, Signature},
    helpers::Byteable,
    server::{
        ServerState,
        handler::{AuroraProtocolCommand, users::UserResponse},
        protocol::AuroraProtocolResponse,
    },
};

#[derive(Debug)]
pub struct Who;

impl AuroraProtocolCommand for Who {
    type RequestPayload = WhoRequest;

    type ResponsePayload = WhoResponse;

    async fn process(
        _: Self::RequestPayload,
        state: &ServerState,
    ) -> AuroraProtocolResponse<Self::ResponsePayload> {
        let user = {
            let config = state.config.read().await;
            let user_pub_key = config.public_key();
            state.repositories.user().get_user(user_pub_key).await
        };

        let (address, address_signature) = {
            let config = state.config.read().await;
            let address = config.eepsite_address().clone();
            let private_key = config.private_key().clone();
            let address_signature = address.sign(&private_key);

            (address, address_signature)
        };

        if let Some(user) = user {
            AuroraProtocolResponse::ok(Self::ResponsePayload {
                user: user.into(),
                address,
                address_signature,
            })
        } else {
            AuroraProtocolResponse::not_found("User not found".to_string())
        }
    }
}

#[derive(Debug, byteable_derive::Byteable)]
pub struct WhoRequest {}

#[derive(Debug, byteable_derive::Byteable)]
pub struct WhoResponse {
    pub user: UserResponse,
    pub address: I2PAddress,
    pub address_signature: Signature,
}
