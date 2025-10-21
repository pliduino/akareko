use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    db::user::User,
    errors::{DecodeError, EncodeError},
    hash::{PublicKey, Signature},
    helpers::Byteable,
};

pub mod get_users;
pub mod who;
pub use get_users::GetUsers;
pub use who::Who;

#[derive(Debug, Clone, byteable_derive::Byteable)]
pub struct UserResponse {
    pub pub_key: PublicKey,
    pub name: String,
    pub signature: Signature,
    pub timestamp: u64,
}

impl UserResponse {
    pub fn as_user(self) -> User {
        User::new(
            self.name,
            self.timestamp,
            self.pub_key,
            self.signature,
            None,
        )
    }
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        let (pub_key, name, timestamp, _, signature, _) = user.as_tuple();

        UserResponse {
            pub_key: pub_key,
            name: name,
            signature: signature,
            timestamp: timestamp,
        }
    }
}
