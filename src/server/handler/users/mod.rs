use crate::{
    db::user::{I2PAddress, User},
    hash::{PublicKey, Signature},
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
    pub address: I2PAddress,
}

impl UserResponse {
    pub fn as_user(self) -> User {
        User::new(
            self.name,
            self.timestamp,
            self.pub_key,
            self.signature,
            self.address,
        )
    }
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        let (pub_key, name, timestamp, address, signature, _) = user.as_tuple();

        UserResponse {
            pub_key,
            name,
            signature,
            timestamp,
            address,
        }
    }
}
