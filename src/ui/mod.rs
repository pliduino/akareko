use anawt::{AlertCategory, SettingsPack, TorrentClient, options::AnawtOptions};
use clap::Parser;
use iced::{
    Length, Subscription, Task, Theme, alignment,
    widget::{Column, Container, button, column, stack, text},
    window,
};
use rclite::Arc;
use std::{collections::BTreeMap, path::PathBuf};
use tokio::sync::{Mutex, RwLock, mpsc};
use tracing::{error, info};
use trayicon::TrayIcon;

use crate::{
    CliArgs,
    config::AkarekoConfig,
    db::{
        FullSyncTarget, Repositories,
        event::make_event_filter,
        index::tags::MangaTag,
        schedule::{Schedule, ScheduleType, Scheduler},
        user::I2PAddress,
    },
    hash::PublicKey,
    helpers::now_timestamp,
    server::{
        AkarekoServer,
        client::{AkarekoClient, TIME_OFFSET, pool::ClientPool},
    },
    ui::{
        components::{
            modal::{Modal, ModalMessage, modal},
            toast::{Toast, ToastType, toast_worker},
        },
        tray::initialize_tray_icon,
        views::{View, ViewMessage, home::HomeView},
    },
};

mod components;
mod icons;
mod style;
mod tray;
mod views;

#[derive(Debug, Clone, PartialEq)]
pub enum TrayIconMessage {
    OpenWindow,
    Exit,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenWindow(WindowType),
    CloseWindow(iced::window::Id),
    Exit,

    RepositoryLoaded(Repositories),
    ConfigLoaded(AkarekoConfig),
    TorrentClientLoaded(TorrentClient),
    ClientLoaded(AkarekoClient),
    DownloadTorrent { magnet: String, path: String },
    ChangeView(View),
    ViewMessage(ViewMessage),
    BackHistory,

    ToastSenderReady(mpsc::Sender<Toast>),
    PostToast(Toast),
    CloseToast(usize),

    ModalMessage(ModalMessage),
    OpenModal(Modal),
    CloseModal,

    SaveTorrent,

    AddSchedule(Schedule),
    RemoveSchedule(Schedule),
    LoadFullSyncAddresses(Vec<(I2PAddress, FullSyncTarget)>),
    TryConsumeSchedule,

    TrayIconMessage(TrayIconMessage),

    Nothing,
}

#[derive(Debug, Clone)]
pub struct LiFo<T, const N: usize> {
    stack: [Option<T>; N],
    last_index: usize,
}

impl<T, const N: usize> LiFo<T, N> {
    pub fn new() -> Self {
        Self {
            stack: [const { None }; N],
            last_index: N - 1,
        }
    }

    pub fn push(&mut self, item: T) {
        self.last_index = (self.last_index + 1) % N;
        self.stack[self.last_index] = Some(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        let item = self.stack[self.last_index].take();
        if item.is_some() {
            self.last_index = (self.last_index + N - 1) % N;
        }
        item
    }

    pub fn can_pop(&self) -> bool {
        self.stack[self.last_index].is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowType {
    Main,
}

pub struct AppState {
    repositories: Option<Repositories>,
    config: AkarekoConfig,
    server_config: Arc<RwLock<AkarekoConfig>>,

    view: View,
    history: LiFo<View, 10>,

    scheduler: Scheduler,

    client_pool: Option<ClientPool>,
    torrent_client: Option<TorrentClient>,

    toast_tx: Option<mpsc::Sender<Toast>>,
    toasts: Vec<Toast>,

    theme: Theme,

    tray_icon: TrayIcon<TrayIconMessage>,

    windows: BTreeMap<window::Id, WindowType>,

    modal: Option<Modal>,
}

impl AppState {
    pub fn new() -> Self {
        let tray_icon = initialize_tray_icon();
        Self {
            repositories: None,
            config: AkarekoConfig::default(),
            client_pool: None,
            server_config: Arc::new(RwLock::new(AkarekoConfig::default())),
            torrent_client: None,
            view: View::Home(HomeView::new()),
            history: LiFo::new(),
            toast_tx: None,
            toasts: Vec::new(),
            theme: Theme::CatppuccinMocha,
            modal: None,
            scheduler: Scheduler::new(),
            tray_icon,
            windows: BTreeMap::new(),
        }
    }

    pub fn boot() -> (AppState, Task<Message>) {
        let args = CliArgs::parse();

        let open_task = match args.minimized {
            true => Task::none(),
            false => Task::done(Message::OpenWindow(WindowType::Main)),
        };
        (
            AppState::new(),
            open_task.chain(Task::perform(AkarekoConfig::load(), |c| {
                Message::ConfigLoaded(c)
            })),
        )
    }

    fn has_initialized(&self) -> bool {
        self.repositories.is_some() && self.client_pool.is_some() && self.torrent_client.is_some()
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub fn view(&self, _id: window::Id) -> iced::Element<'_, Message> {
        if !self.has_initialized() {
            return column![text("Loading...")].into();
        }

        let mut back = button(text("Back"));

        if self.history.can_pop() {
            back = back.on_press(Message::BackHistory);
        }

        let base = column![back, View::view(self)]
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);

        let base = if self.modal.is_some() {
            modal(base, Modal::view(self), Message::CloseModal)
        } else {
            base.into()
        };

        let toasts = self
            .toasts
            .iter()
            .rev()
            .enumerate()
            .map(|(i, t)| t.view(i))
            .collect();

        stack![
            base,
            Container::new(Column::from_vec(toasts).align_x(alignment::Horizontal::Right))
                .align_right(Length::Fill)
                .align_bottom(Length::Fill)
        ]
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Exit => return iced::exit(),
            Message::ConfigLoaded(c) => {
                self.config = c.clone();

                // Nothing is using it here as it's still in the initialization process so it's ok to use blocking_write
                {
                    let mut server_config = self.server_config.blocking_write();
                    *server_config = c;
                }

                let config = self.config.clone();

                return Task::batch([
                    Task::perform(AkarekoClient::new(config.clone()), |c| {
                        Message::ClientLoaded(c)
                    }),
                    Task::future(async move {
                        info!("Initializing Repositories...");
                        let r = Repositories::initialize(&config).await;
                        Message::RepositoryLoaded(r)
                    }),
                    Task::future(async move {
                        let mut settings_pack = SettingsPack::new();
                        settings_pack.set_alert_mask(
                            AlertCategory::Error | AlertCategory::Storage | AlertCategory::Status,
                        );

                        let client =
                            TorrentClient::create(AnawtOptions::new().settings_pack(settings_pack));

                        // TODO: this should not kill the client
                        match client.load("./data/torrents".into()).await {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Failed to load torrents: {}", e);
                                return Message::PostToast(Toast {
                                    title: "Failed to load torrents".to_string(),
                                    body: e.to_string(),
                                    ty: ToastType::Error,
                                });
                            }
                        }

                        Message::TorrentClientLoaded(client)
                    }),
                ]);
            }
            Message::RepositoryLoaded(r) => {
                self.repositories = Some(r.clone());

                let server_config = self.server_config.clone();
                let server_repo = r.clone();
                tokio::spawn(async move {
                    let server = AkarekoServer::new();
                    server
                        .run(server_config.clone(), server_repo)
                        .await
                        .unwrap();
                });
                return Task::future(async move {
                    let targets = r.full_sync_addresses().await.unwrap();
                    let pub_keys = targets
                        .iter()
                        .map(|t| t.pub_key.clone())
                        .collect::<Vec<_>>();

                    let users = r.user().get_users(pub_keys).await.unwrap();

                    let addresses: Vec<(I2PAddress, FullSyncTarget)> = users
                        .into_iter()
                        .zip(targets)
                        .map(|(u, t)| (u.into_address(), t))
                        .collect();

                    Message::LoadFullSyncAddresses(addresses)
                });
            }
            Message::TorrentClientLoaded(t) => {
                self.torrent_client = Some(t);
            }
            Message::ClientLoaded(client) => {
                self.client_pool = Some(ClientPool::new(
                    client,
                    self.config.max_client_connections() as u16,
                ));
            }
            Message::DownloadTorrent { magnet, path } => {
                if let Some(torrent_client) = &self.torrent_client {
                    let client = torrent_client.clone();

                    return Task::future(async move {
                        let _ = client.add_magnet(&magnet, &path).await;
                        Message::Nothing
                    });
                }
            }
            Message::ChangeView(v) => {
                let old_view = std::mem::replace(&mut self.view, v);
                self.history.push(old_view);
                return View::on_enter(self);
            }
            Message::ViewMessage(m) => {
                return View::update(m, self);
            }
            Message::ModalMessage(m) => {
                return Modal::update(m, self);
            }
            Message::BackHistory => {
                if let Some(v) = self.history.pop() {
                    self.view = v;
                    return View::on_enter(self);
                }
            }
            Message::ToastSenderReady(tx) => {
                if self.toast_tx.is_some() {
                    error!("Tried to set ToastSenderReady twice");
                } else {
                    self.toast_tx = Some(tx);
                }
            }
            Message::PostToast(toast) => {
                self.add_toast(toast);
            }
            Message::CloseToast(i) => {
                self.toasts.remove(i);
            }
            Message::OpenModal(m) => {
                self.modal = Some(m);
            }
            Message::CloseModal => {
                self.close_modal();
            }
            Message::SaveTorrent => {
                if let Some(client) = &self.torrent_client {
                    let client = client.clone();
                    return Task::future(async move {
                        client.save(PathBuf::from("./data/torrents")).await.unwrap();
                        Message::Nothing
                    });
                }
            }
            Message::OpenWindow(window_type) => {
                match window_type {
                    WindowType::Main => {
                        if self.windows.values().any(|v| *v == window_type) {
                            return Task::done(Message::Nothing);
                        }
                    }
                }

                let (id, task) = window::open(window::Settings {
                    size: iced::Size::new(800.0, 600.0),
                    maximized: true,
                    exit_on_close_request: false,
                    ..Default::default()
                });

                self.windows.insert(id, window_type);

                return task.map(|_| Message::Nothing);
            }
            Message::CloseWindow(id) => {
                let window_type = self.windows.remove(&id).unwrap();
                return window::close(id);
            }
            Message::AddSchedule(schedule) => {
                self.scheduler.schedule(schedule);
            }
            Message::RemoveSchedule(schedule) => {
                self.scheduler.remove(schedule);
            }
            Message::TryConsumeSchedule => {
                let (Some(pool), Some(db)) = (self.client_pool.clone(), self.repositories.clone())
                else {
                    return Task::none();
                };
                let Some(schedule) = self.scheduler.try_next() else {
                    return Task::none();
                };

                info!("Consuming schedule: {schedule:?}");

                let scheduler_config = self.config.scheduler_config().clone();
                return Task::future(async move {
                    let mut client = pool.get_client().await;
                    let (server_timestamp, increment) = match schedule.schedule_type {
                        ScheduleType::FullSync(ref pub_key) => {
                            let server_timestamp = match client
                                .sync_events(&schedule.address, schedule.last_sync, &db)
                                .await
                            {
                                Ok(t) => t,
                                Err(e) => {
                                    error!("Failed to sync events: {}", e);
                                    return Message::AddSchedule(Schedule {
                                        when: now_timestamp() + scheduler_config.full_sync_interval,
                                        address: schedule.address,
                                        schedule_type: schedule.schedule_type,
                                        last_sync: schedule.last_sync,
                                    });
                                }
                            };

                            db.upsert_full_sync_address(FullSyncTarget {
                                pub_key: pub_key.clone(),
                                last_sync: server_timestamp,
                            })
                            .await
                            .unwrap();

                            (server_timestamp, scheduler_config.full_sync_interval)
                        }
                        ScheduleType::SyncMangaContent(ref hash) => {
                            let filter = db
                                .index()
                                .make_filter::<MangaTag>(
                                    &hash,
                                    if schedule.last_sync < TIME_OFFSET {
                                        schedule.last_sync - TIME_OFFSET
                                    } else {
                                        0
                                    },
                                )
                                .await
                                .unwrap();

                            client
                                .get_manga_content(
                                    &schedule.address,
                                    db.index(),
                                    hash.clone(),
                                    schedule.last_sync,
                                    Some(filter),
                                )
                                .await
                                .unwrap();

                            (0, 0)
                        }
                        ScheduleType::SyncPost(ref topic) => {
                            let filter = db
                                .posts()
                                .make_filter(
                                    topic.clone(),
                                    if schedule.last_sync < TIME_OFFSET {
                                        schedule.last_sync - TIME_OFFSET
                                    } else {
                                        0
                                    },
                                )
                                .await
                                .unwrap();

                            (0, 0)
                        }
                    };

                    Message::AddSchedule(Schedule {
                        when: now_timestamp() + increment,
                        address: schedule.address,
                        schedule_type: schedule.schedule_type,
                        last_sync: server_timestamp,
                    })
                });
            }
            Message::LoadFullSyncAddresses(a) => {
                for (address, target) in a {
                    self.scheduler.schedule(Schedule {
                        when: target.last_sync + self.config.scheduler_config().full_sync_interval,
                        last_sync: target.last_sync,
                        address,
                        schedule_type: ScheduleType::FullSync(target.pub_key),
                    });
                }
            }
            Message::TrayIconMessage(m) => match m {
                TrayIconMessage::OpenWindow => {
                    return Task::done(Message::OpenWindow(WindowType::Main));
                }
                TrayIconMessage::Exit => {
                    return Task::done(Message::Exit);
                }
            },
            Message::Nothing => {}
        }

        Task::none()
    }

    pub fn add_toast(&mut self, toast: Toast) {
        self.toasts.push(toast);
    }

    pub fn close_modal(&mut self) {
        self.modal = None;
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        let toast_subscription = Subscription::run(toast_worker);
        let view_subscription = self.view.subscription();

        let tray_icon_subscription = self.tray_icon.subscribe();

        Subscription::batch([
            iced::time::every(std::time::Duration::from_millis(500)).map(|_| Message::Nothing),
            iced::time::every(std::time::Duration::from_millis(3500))
                .map(|_| Message::TryConsumeSchedule),
            toast_subscription,
            view_subscription,
            window::close_requests().map(Message::CloseWindow),
            tray_icon_subscription.map(|m| Message::TrayIconMessage(m)),
        ])
    }
}
