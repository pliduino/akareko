use anawt::{InfoHash, RemoveFlags};
use freya::{prelude::*, query::*, radio::RadioStation};

use crate::{
    db::Magnet,
    errors::TorrentError,
    ui::{
        AppChannel, AppState, ResourceState,
        queries::{FetchTorrentStatus, FetchTorrentWatchers},
    },
};

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct RemoveTorrent;

impl MutationCapability for RemoveTorrent {
    type Ok = ();
    type Err = TorrentError;
    type Keys = (InfoHash, RemoveFlags);

    async fn run(&self, keys: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let radio = try_consume_root_context::<RadioStation<AppState, AppChannel>>();
        let Some(radio) = radio else {
            return Err(TorrentError::NotInitialized);
        };

        match &radio.read().torrent_client {
            ResourceState::Loaded(c) => c
                .remove_torrent(keys.0, keys.1)
                .await
                .map_err(|_| TorrentError::Unknown),
            _ => Err(TorrentError::NotInitialized),
        }
    }

    async fn on_settled(&self, keys: &Self::Keys, result: &Result<Self::Ok, Self::Err>) {
        if result.is_ok() {
            QueriesStorage::<FetchTorrentStatus>::invalidate_matching(keys.0).await;
            QueriesStorage::<FetchTorrentWatchers>::invalidate_all().await;
        }
    }
}
