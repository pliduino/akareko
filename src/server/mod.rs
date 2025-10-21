use std::io;

use rclite::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use yosemite::{Session, SessionOptions, style};

use crate::{
    config::AuroraConfig,
    db::{Repositories, user::UserRepository},
    errors::{DecodeError, IoError, ServerError},
    helpers::{Byteable, b32_from_pub_b64},
    server::protocol::AuroraProtocolVersion,
};

pub mod client;
mod handler;
pub mod protocol;
pub mod proxy;

pub struct AuroraServer {}

// #[cfg(target_os = "windows")]
// fn get_i2p_config_folder() -> PathBuf {
//     dirs::config_dir().unwrap().join("i2pd")
// }

// #[cfg(target_os = "linux")]
// fn get_i2p_config_folder() -> PathBuf {
//     dirs::config_dir().unwrap().join("i2pd")
// }

// #[cfg(target_os = "android")]
// fn get_i2p_config_folder() -> PathBuf {
//     dirs::config_dir().unwrap().join("i2pd")
// }

#[derive(Clone)]
struct ServerState {
    pub config: Arc<RwLock<AuroraConfig>>,
    pub repositories: Repositories,
}

impl AuroraServer {
    pub fn new() -> AuroraServer {
        AuroraServer {}
    }

    pub async fn run(
        &self,
        config: Arc<RwLock<AuroraConfig>>,
        repositories: Repositories,
    ) -> Result<(), ServerError> {
        info!("Starting server SAMv3 session");

        let mut sam_session = {
            let config_guard = config.read().await;

            Session::<style::Stream>::new(SessionOptions {
                // nickname: "AuroraServer".to_string(),
                samv3_tcp_port: config_guard.sam_port(),
                destination: yosemite::DestinationKind::Persistent {
                    private_key: config_guard.eepsite_key().clone(),
                },
                ..Default::default()
            })
            .await?
        };

        info!("Server Started");
        // info!(
        //     "Starting server on {}",
        //     b64_to_b32_i2p(sam_session.destination()).unwrap()
        // );

        let state = ServerState {
            config,
            repositories,
        };

        while let Ok(mut stream) = sam_session.accept().await {
            let state = state.clone();
            tokio::spawn(async move {
                let address = stream.remote_destination();

                // state.repositories.user().get_user(address);

                loop {
                    let version = match AuroraProtocolVersion::decode(&mut stream).await {
                        Ok(v) => v,
                        Err(e) => match e {
                            DecodeError::IoError(e) => {
                                match e.kind() {
                                    io::ErrorKind::UnexpectedEof => {
                                        //
                                    }
                                    _ => {
                                        error!("Failed to decode version: {}", e);
                                    }
                                }
                                break;
                            }
                            _ => {
                                error!("Failed to decode version: {}", e);
                                break;
                            }
                        },
                    };

                    match version {
                        AuroraProtocolVersion::V1 => {
                            handler::V1::handle(&mut stream, &state).await;
                        }
                    }
                }
            });
        }

        Ok(())
    }
}
