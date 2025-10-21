use std::{
    collections::{HashMap, HashSet},
    ops::DerefMut,
};

use rclite::Arc;
use tokio::{
    io::AsyncWriteExt,
    sync::{Mutex, MutexGuard},
};
use tracing::{error, info};
use yosemite::{Session, SessionOptions, Stream, style};

use crate::{
    config::AuroraConfig,
    db::{
        Content, Index, IndexTag, Repositories, TaggedIndex,
        index::{NovelTag, TaggedContent},
        user::{I2PAddress, User, UserRepository},
    },
    errors::ClientError,
    hash::{Hash, PublicKey, Signable},
    helpers::Byteable,
    server::{
        handler::{
            self, AuroraProtocolCommand, AuroraProtocolCommandMetadata,
            index::{ExchangeContentRequest, GetAllIndexesRequest, GetIndexesRequest},
            users::{get_users::GetUsersRequest, who::WhoRequest},
        },
        protocol::AuroraProtocolResponse,
        proxy::LoggingStream, // proxy::I2PConnector,
    },
};

#[derive(Clone)]
pub struct AuroraClient {
    repositories: Repositories,
    session: Arc<Mutex<Session<style::Stream>>>,
}

impl AuroraClient {
    pub async fn new(repositories: Repositories, config: AuroraConfig) -> Self {
        info!("Initializing AuroraClient...");

        let session = Arc::new(Mutex::new(
            Session::<style::Stream>::new(SessionOptions {
                // nickname: "AuroraClient".to_string(),
                samv3_tcp_port: config.sam_port(),
                destination: yosemite::DestinationKind::Persistent {
                    private_key: config.eepsite_key().clone(),
                },
                ..Default::default()
            })
            .await
            .unwrap(),
        ));

        info!("Initialized AuroraClient");

        Self {
            repositories,
            session,
        }
    }

    async fn get_stream(&mut self, url: &I2PAddress) -> Result<LoggingStream<Stream>, ClientError> {
        let session = self.session.clone();
        let stream = session.lock().await.connect(url.inner()).await?;

        let stream = LoggingStream(stream);

        Ok(stream)
    }

    // ╔===========================================================================╗
    // ║                                   Index                                   ║
    // ╚===========================================================================╝

    pub async fn get_all_indexes<T: IndexTag>(
        &mut self,
        url: &I2PAddress,
    ) -> Result<Vec<Index<T>>, ClientError> {
        let mut stream = self.get_stream(url).await?;

        let res = handler::index::GetAllIndexes::request(
            GetAllIndexesRequest {
                tag: T::TAG.to_string(),
            },
            &mut stream,
        )
        .await?;

        if !res.status().is_ok() {
            return Err(ClientError::UnexpectedResponseCode {
                status: res.status().clone(),
            });
        }

        let Some(payload) = res.payload() else {
            return Err(ClientError::MissingPayload);
        };

        let mut indexes: Vec<Index<T>> = Vec::with_capacity(payload.indexes.len());

        for index in payload.indexes {
            match index {
                TaggedIndex::Novel(index) => {
                    if T::TAG == NovelTag::TAG {
                        indexes.push(index.transmute());
                    }
                }
            }
        }

        Ok(indexes)
    }

    // ╔===========================================================================╗
    // ║                                 Exchange                                  ║
    // ╚===========================================================================╝

    pub async fn routine_exchange(&mut self, url: &I2PAddress) -> Result<(), ClientError> {
        let mut stream = self.get_stream(url).await?;

        let who = Self::who_internal(&mut stream, url).await?;

        self.repositories.user().upsert_user(who).await?;

        let response = handler::index::ExchangeContent::request(
            ExchangeContentRequest { count: 10 },
            &mut stream,
        )
        .await?;

        let contents = response.payload_if_ok()?.contents;

        let mut existing_indexes: HashSet<Hash> = HashSet::new();
        let mut missing_indexes: Vec<(String, Hash)> = Vec::new();

        for content in contents.iter() {
            match content {
                TaggedContent::Novel(content) => {
                    match self
                        .repositories
                        .index()
                        .get_index::<NovelTag>(content.index_hash())
                        .await
                    {
                        Ok(i) => match i {
                            Some(_) => {
                                existing_indexes.insert(content.index_hash().clone());
                            }
                            None => {
                                missing_indexes.push((
                                    NovelTag::TAG.to_string(),
                                    content.index_hash().clone(),
                                ));
                            }
                        },
                        Err(e) => {
                            error!("Failed to get index: {}", e);
                        }
                    }
                }
            }
        }

        let response = handler::index::GetIndexes::request(
            GetIndexesRequest {
                indexes: missing_indexes,
            },
            &mut stream,
        )
        .await?
        .payload_if_ok()?;

        for index in response.indexes {
            match index {
                TaggedIndex::Novel(index) => {
                    match self.repositories.index().add_index(index).await {
                        Ok(i) => {
                            existing_indexes.insert(i.hash().clone());
                        }
                        Err(e) => {
                            error!("Failed to add index: {}", e);
                        }
                    }
                }
            }
        }

        for content in contents.into_iter() {
            if !existing_indexes.contains(content.index_hash()) {
                continue;
            }

            match content {
                TaggedContent::Novel(content) => {
                    match self.repositories.index().add_content(content).await {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Failed to add content: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // ╔===========================================================================╗
    // ║                                   User                                    ║
    // ╚===========================================================================╝

    async fn who_internal(
        stream: &mut LoggingStream<Stream>,
        url: &I2PAddress,
    ) -> Result<User, ClientError> {
        let res = handler::users::Who::request(WhoRequest {}, stream).await?;

        if !res.status().is_ok() {
            return Err(ClientError::UnexpectedResponseCode {
                status: res.status().clone(),
            });
        }

        let Some(payload) = res.payload() else {
            return Err(ClientError::MissingPayload);
        };

        if !payload
            .address
            .verify(&payload.user.pub_key, &payload.address_signature)
        {
            return Err(ClientError::InvalidSignature);
        }

        let mut user = payload.user.as_user();

        user.set_address(Some(url.clone()));

        Ok(user)
    }

    pub async fn who(&mut self, url: &I2PAddress) -> Result<User, ClientError> {
        let mut stream = self.get_stream(url).await?;

        Self::who_internal(&mut stream, url).await
    }

    pub async fn request_users(
        &mut self,
        url: &I2PAddress,
        pub_keys: Vec<PublicKey>,
    ) -> Result<Vec<User>, ClientError> {
        let mut stream = self.get_stream(url).await?;

        let res =
            handler::users::GetUsers::request(GetUsersRequest { pub_keys }, &mut stream).await?;

        if !res.status().is_ok() {
            return Err(ClientError::UnexpectedResponseCode {
                status: res.status().clone(),
            });
        }

        let Some(payload) = res.payload() else {
            return Err(ClientError::MissingPayload);
        };

        let users: Vec<User> = payload.users.into_iter().map(|u| u.as_user()).collect();

        // TODO
        // self.repositories
        //     .get_user_repository()
        //     .save_users(users.clone())
        //     .await?;

        Ok(users)
    }
}

impl std::fmt::Debug for AuroraClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuroraClient").finish()
    }
}
