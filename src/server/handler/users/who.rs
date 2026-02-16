use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    db::{
        Repositories,
        user::{I2PAddress, User, UserRepository},
    },
    errors::{DecodeError, EncodeError},
    hash::{PrivateKey, Signable, Signature},
    helpers::{Byteable, now_timestamp},
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
        req: Self::RequestPayload,
        state: &ServerState,
    ) -> AuroraProtocolResponse<Self::ResponsePayload> {
        let response: Option<WhoResponse> = {
            let config = state.config.read().await;
            let user_pub_key = config.public_key();
            let priv_key = config.private_key();
            match state.repositories.user().await.get_user(user_pub_key).await {
                Some(user) => Some(WhoResponse::new_signed(
                    user,
                    &req.request_address,
                    priv_key,
                )),
                None => None,
            }
        };

        if let Some(response) = response {
            AuroraProtocolResponse::ok(response)
        } else {
            AuroraProtocolResponse::not_found("User not found".to_string())
        }
    }
}

#[derive(Debug, byteable_derive::Byteable)]
pub struct WhoRequest {
    pub request_address: I2PAddress,
}

#[derive(Debug, byteable_derive::Byteable)]
pub struct WhoResponse {
    pub user: UserResponse,
    pub timestamp: u64,
    pub signature: Signature, // Timestamp + Address of requesting user
}

impl WhoResponse {
    pub fn verification_bytes(&self, request_address: &I2PAddress) -> Vec<u8> {
        let mut bytes = self.timestamp.to_le_bytes().to_vec();
        bytes.extend(request_address.inner().as_bytes());
        bytes
    }

    pub fn new_signed(user: User, request_address: &I2PAddress, priv_key: &PrivateKey) -> Self {
        let mut response = Self {
            user: user.into(),
            timestamp: now_timestamp(),
            signature: Signature::empty(),
        };

        let to_sign = response.verification_bytes(request_address);
        response.signature = priv_key.sign(&to_sign);

        response
    }

    pub fn verify(&self, request_address: &I2PAddress) -> bool {
        let bytes = self.verification_bytes(request_address);
        self.user.pub_key.verify(&bytes, &self.signature)
    }
}
