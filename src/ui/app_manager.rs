use anawt::{TorrentClient, options::AnawtOptions};
use emissary_core::{Config, Ntcp2Config, SamConfig, Ssu2Config, TransitConfig, router::Router};
use emissary_util::{
    reseeder::Reseeder,
    runtime::tokio::Runtime,
    storage::{Storage, StorageBundle},
};
use freya::radio::RadioStation;
use tokio::sync::RwLock;
use tracing::error;

use crate::{
    config::AkarekoConfig,
    db::Repositories,
    server::{
        AkarekoServer,
        client::{AkarekoClient, pool::ClientPool},
    },
    ui::{AppChannel, AppState, ResourceState},
};

pub enum Event {
    RemoveMainWindow,
}

enum LoadEvent {
    LoadedClient(ClientPool),
}

pub struct AppManager {
    client_thread: Option<tokio::task::JoinHandle<()>>,
    radio_station: RadioStation<AppState, AppChannel>,
    load_tx: tokio::sync::mpsc::UnboundedSender<LoadEvent>,
    load_rx: tokio::sync::mpsc::UnboundedReceiver<LoadEvent>,
    rx: tokio::sync::mpsc::UnboundedReceiver<Event>,
}

impl AppManager {
    pub async fn run_manager(mut self) {
        self.radio_station.write_channel(AppChannel::Config).config = ResourceState::Loading;
        let config = AkarekoConfig::load().await;
        self.radio_station.write_channel(AppChannel::Config).config =
            ResourceState::Loaded(config.clone());

        let storage = Storage::new::<Runtime>(None).await.unwrap();
        let StorageBundle {
            ntcp2_iv,
            ntcp2_key,
            profiles,
            router_info,
            mut routers,
            signing_key,
            static_key,
            ssu2_intro_key,
            ssu2_static_key,
        } = storage.load().await;

        if routers.is_empty() {
            match Reseeder::reseed::<Runtime>(None, false).await {
                Ok(reseed_routers) => {
                    for info in reseed_routers {
                        let _ = storage
                            .store_router_info(info.name.to_string(), info.router_info.clone())
                            .await;
                        routers.push(info.router_info);
                    }
                }

                Err(error) => tracing::error!(
                    num_routers = routers.len(),
                    ?error,
                    "failed to reseed router",
                ),
            }
        }

        let i2p_router_config = Config {
            ntcp2: Some(Ntcp2Config {
                port: 25515,
                key: ntcp2_key,
                iv: ntcp2_iv,
                publish: true,
                ipv4_host: None,
                ipv6_host: None,
                ipv4: true,
                ipv6: true,
                ml_kem: Some(4),
                disable_pq: false,
            }),
            ssu2: Some(Ssu2Config {
                intro_key: ssu2_intro_key,
                static_key: ssu2_static_key,
                ipv4: true,
                ipv4_host: None,
                ipv6: true,
                ipv6_host: None,
                port: 25515,
                publish: true,
                ipv4_mtu: None,
                ipv6_mtu: None,
                disable_pq: false,
                ml_kem: Some("4".to_string()),
            }),
            samv3_config: Some(SamConfig {
                tcp_port: config.sam_port(),
                udp_port: config.sam_port(),
                host: "127.0.0.1".to_string(),
            }),
            routers,
            profiles,
            router_info,
            static_key: Some(static_key),
            signing_key: Some(signing_key),
            transit: Some(TransitConfig {
                max_tunnels: Some(1000),
            }),
            ..Config::default()
        };

        let (router, _events, router_info) = Router::<Runtime>::new(
            i2p_router_config,
            None,
            Some(std::sync::Arc::new(storage.clone())),
        )
        .await
        .map_err(|error| panic!("failed to start router: {error}"))
        .unwrap();

        tokio::spawn(router);

        storage.store_local_router_info(router_info).await.unwrap();

        self.radio_station
            .write_channel(AppChannel::TorrentClient)
            .torrent_client = ResourceState::Loading;
        let torrent_client = TorrentClient::create(AnawtOptions::new());
        match torrent_client.load("./data/torrents".into()).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to load torrents: {}", e);
            }
        }
        self.radio_station
            .write_channel(AppChannel::TorrentClient)
            .torrent_client = ResourceState::Loaded(torrent_client);

        self.radio_station
            .write_channel(AppChannel::Repository)
            .repositories = ResourceState::Loading;
        let repos = Repositories::initialize(&config).await;
        self.radio_station
            .write_channel(AppChannel::Repository)
            .repositories = ResourceState::Loaded(repos.clone());

        self.radio_station.write_channel(AppChannel::Server).server = ResourceState::Loading;
        let server = AkarekoServer::new();
        let server_conf = rclite::Arc::new(RwLock::new(config.clone()));
        tokio::spawn(async move {
            server.run(server_conf, repos).await.unwrap();
        });
        self.radio_station.write_channel(AppChannel::Server).server = ResourceState::Loaded(());

        self.start_client_thread();

        self.process_events().await;
    }

    pub fn new(
        radio_station: RadioStation<AppState, AppChannel>,
    ) -> (AppManager, tokio::sync::mpsc::UnboundedSender<Event>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let (load_tx, load_rx) = tokio::sync::mpsc::unbounded_channel();

        let mut manager = AppManager {
            client_thread: None,
            radio_station,
            load_tx,
            load_rx,
            rx,
        };

        manager.start_client_thread();

        (manager, tx)
    }

    pub fn start_client_thread(&mut self) {
        if let Some(t) = self.client_thread.take() {
            t.abort();
        };

        let config = match self.radio_station.read().config {
            ResourceState::Loaded(ref config) => config.clone(),
            _ => return,
        };

        self.radio_station.write_channel(AppChannel::Client).client = ResourceState::Loading;

        let load_tx = self.load_tx.clone();
        self.client_thread = Some(tokio::spawn(async move {
            let client = ClientPool::new(
                AkarekoClient::new(config.clone()).await,
                config.max_client_connections() as u16,
            );

            load_tx.send(LoadEvent::LoadedClient(client)).unwrap();
        }));
    }

    pub async fn process_events(&mut self) {
        loop {
            tokio::select! {
                val = self.rx.recv() => {
                    match val.unwrap() {
                        Event::RemoveMainWindow => {
                            self.radio_station.write_channel(AppChannel::Window).windows_state.remove_main_window();
                        },
                    }
                }
                val = self.load_rx.recv() => {
                    match val.unwrap() {
                        LoadEvent::LoadedClient(client) => {
                            self.radio_station.write_channel(AppChannel::Client).client =
                                ResourceState::Loaded(client);
                            self.client_thread = None;
                        }
                    }
                }
            }
        }
    }
}
