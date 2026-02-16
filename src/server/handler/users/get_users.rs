use crate::{
    db::{Repositories, user::UserRepository},
    hash::PublicKey,
    server::{
        ServerState,
        handler::{AuroraProtocolCommand, users::UserResponse},
        protocol::AuroraProtocolResponse,
    },
};

pub struct GetUsers;

impl AuroraProtocolCommand for GetUsers {
    type RequestPayload = GetUsersRequest;
    type ResponsePayload = GetUsersResponse;

    async fn process(
        req: Self::RequestPayload,
        state: &ServerState,
    ) -> AuroraProtocolResponse<Self::ResponsePayload> {
        let users = match state
            .repositories
            .user()
            .await
            .get_users(req.pub_keys)
            .await
        {
            Ok(users) => users,
            Err(_) => {
                return AuroraProtocolResponse::internal_error("Failed to get users".to_string());
            }
        };

        let users = users.into_iter().map(|u| u.into()).collect();

        AuroraProtocolResponse::ok(Self::ResponsePayload { users })
    }
}

#[derive(Debug, byteable_derive::Byteable)]
pub struct GetUsersRequest {
    pub pub_keys: Vec<PublicKey>,
}

#[derive(Debug, byteable_derive::Byteable)]
pub struct GetUsersResponse {
    pub users: Vec<UserResponse>,
}
