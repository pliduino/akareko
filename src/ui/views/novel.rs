use anawt::{AnawtTorrentStatus, InfoHash, TorrentState};
use iced::{
    Task,
    widget::{Column, button, row, text},
};
use tokio::sync::watch;
use tracing::info;

use crate::{
    db::{Content, Index, index::NovelTag},
    helpers::SanitizedString,
    ui::{
        AppState, Message,
        views::{
            View, ViewMessage, add_chapter::AddNovelChapterView, image_viewer::ImageViewerView,
        },
    },
};

#[derive(Debug, Clone)]
pub struct NovelView {
    novel: Index<NovelTag>,
    chapters: Vec<Content<NovelTag>>,
    pub torrents: Vec<Option<watch::Receiver<AnawtTorrentStatus>>>,
}

#[derive(Debug, Clone)]
pub enum NovelMessage {
    ChaptersLoaded(Vec<Content<NovelTag>>),
    LoadedTorrentWatcher(Vec<Option<watch::Receiver<AnawtTorrentStatus>>>),
    ReloadTorrents,
    DownloadTorrentAndReload { magnet: String, path: String },
    TorrentStatusUpdated,
}

impl From<NovelMessage> for Message {
    fn from(m: NovelMessage) -> Self {
        Message::ViewMessage(ViewMessage::Novel(m))
    }
}

impl NovelView {
    pub fn new(novel: Index<NovelTag>) -> Self {
        Self {
            novel,
            chapters: vec![],
            torrents: Vec::new(),
        }
    }

    pub fn on_enter(state: &mut AppState) -> Task<Message> {
        if let View::Novel(v) = &mut state.view {
            if let Some(repositories) = &state.repositories {
                let repositories = repositories.clone();
                let novel_hash = v.novel.hash().clone();
                return Task::future(async move {
                    let chapters = repositories.index().get_contents(novel_hash).await;
                    NovelMessage::ChaptersLoaded(chapters).into()
                });
            }
        }
        Task::none()
    }

    pub fn view(&self, state: &AppState) -> iced::Element<Message> {
        let mut column: Vec<iced::Element<Message>> = vec![text(self.novel.title().clone()).into()];

        column.push(
            button(text("Add Chapter"))
                .on_press(Message::ChangeView(View::AddChapter(
                    AddNovelChapterView::new(self.novel.clone()),
                )))
                .into(),
        );

        for i in 0..self.chapters.len() {
            let chapter = &self.chapters[i];
            let rx = self.torrents[i].as_ref();
            match rx {
                Some(rx) => {
                    let status = rx.borrow();

                    match &status.state {
                        TorrentState::Finished | TorrentState::Seeding => {
                            for e in chapter.entries() {
                                column.push(
                                    row![
                                        button(text(e.title.clone())).on_press(
                                            Message::ChangeView(View::ImageViewer(
                                                ImageViewerView::new(
                                                    format!(
                                                        "./data/novel/{}/{}/{}",
                                                        SanitizedString::new(self.novel.title())
                                                            .as_str(),
                                                        chapter.signature().as_base64(),
                                                        chapter.entries()[i].path
                                                    )
                                                    .into()
                                                ),
                                            ))
                                        )
                                    ]
                                    .into(),
                                );
                            }
                            // column.push(row![button(text(chapter.title.clone()))].into());
                        }
                        _ => {
                            column.push(
                                row![
                                    // button(text(chapter.title.clone())),
                                    text(format!("Downloading: {:.1}", status.progress * 100.0))
                                ]
                                .into(),
                            );
                        }
                    }
                }
                None => {
                    column.push(
                        button(text("Download"))
                            .on_press(
                                NovelMessage::DownloadTorrentAndReload {
                                    magnet: chapter.magnet_link.clone().0,
                                    path: format!(
                                        "./data/novel/{}/{}",
                                        SanitizedString::new(self.novel.title()).as_str(),
                                        chapter.signature().as_base64()
                                    ),
                                }
                                .into(),
                            )
                            .into(),
                    );
                }
            }
        }

        Column::from_vec(column).into()
    }

    pub fn update(m: NovelMessage, state: &mut AppState) -> Task<Message> {
        if let View::Novel(v) = &mut state.view {
            match m {
                NovelMessage::ChaptersLoaded(chapters) => {
                    v.torrents = vec![None; chapters.len()];
                    v.chapters = chapters;
                    return Task::done(NovelMessage::ReloadTorrents.into());
                }
                NovelMessage::LoadedTorrentWatcher(watchers) => {
                    info!("Loaded torrent watcher");
                    v.torrents = watchers;
                }
                NovelMessage::DownloadTorrentAndReload { magnet, path } => {
                    info!("Downloading and reloading: {}", magnet);
                    return Task::done(Message::DownloadTorrent { magnet, path })
                        .chain(Task::done(NovelMessage::ReloadTorrents.into()));
                }
                NovelMessage::ReloadTorrents => {
                    info!("Reloading torrents");
                    let torrent_client = state.torrent_client.clone();
                    if let Some(torrent_client) = torrent_client {
                        let chapters = v.chapters.clone();
                        let len = chapters.len();
                        return Task::future(async move {
                            let mut watchers = vec![None; len];

                            for (i, chapter) in chapters.iter().enumerate() {
                                let info_hash = match InfoHash::from_magnet(&chapter.magnet_link.0)
                                {
                                    Ok(info_hash) => info_hash,
                                    Err(_) => continue, // TODO: Invalid magnet, issue chapter deletion
                                };
                                let rx = torrent_client.subscribe_torrent(info_hash).await;
                                watchers[i] = rx;
                            }

                            NovelMessage::LoadedTorrentWatcher(watchers).into()
                        });
                    }
                }
                NovelMessage::TorrentStatusUpdated => {}
            }
        }

        Task::none()
    }
}
