use iced::{
    Task,
    widget::{button, checkbox, column, row, text},
};
use tracing::info;

use crate::{
    config::AuroraConfig,
    db::Repositories,
    ui::{
        AppState, Message,
        components::toast::{Toast, ToastType},
        views::{NovelListView, View, ViewMessage},
    },
};

#[derive(Debug, Clone)]
pub struct SettingsView {
    config: AuroraConfig,
    dirty: bool,
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    UpdateRelay(bool),
    UpdatedDevMode(bool),
    SaveConfig,
    SavedConfig(AuroraConfig),
}

impl From<SettingsMessage> for Message {
    fn from(m: SettingsMessage) -> Self {
        Message::ViewMessage(ViewMessage::Settings(m))
    }
}

impl SettingsView {
    pub fn new() -> Self {
        Self {
            config: AuroraConfig::default(),
            dirty: false,
        }
    }

    pub fn on_enter(state: &mut AppState) -> Task<Message> {
        if let View::Settings(v) = &mut state.view {
            v.config = state.config.clone();
        }
        Task::none()
    }

    pub fn view(&self, state: &AppState) -> iced::Element<Message> {
        let pub_key = self.config.public_key().to_base64();

        let priv_key = self.config.private_key().to_base64();

        let save_message = if self.dirty {
            Some(SettingsMessage::SaveConfig.into())
        } else {
            None
        };

        column![
            row![text("Public Key: "), text(pub_key)],
            row![text("Private Key: "), text(priv_key)],
            row![
                text("Relay: "),
                checkbox("", self.config.is_relay())
                    .on_toggle(|b| { SettingsMessage::UpdateRelay(b).into() }),
            ],
            row![
                text("Dev Mode: "),
                checkbox("", self.config.dev_mode())
                    .on_toggle(|b| { SettingsMessage::UpdatedDevMode(b).into() }),
            ],
            button(text("Save")).on_press_maybe(save_message),
        ]
        .into()
    }

    pub fn update(m: SettingsMessage, state: &mut AppState) -> Task<Message> {
        if let View::Settings(v) = &mut state.view {
            match m {
                SettingsMessage::UpdateRelay(is_relay) => {
                    v.dirty = true;
                    v.config.set_is_relay(is_relay)
                }
                SettingsMessage::UpdatedDevMode(dev_mode) => {
                    v.dirty = true;
                    v.config.set_dev_mode(dev_mode)
                }
                SettingsMessage::SaveConfig => {
                    let config_to_save = v.config.clone();
                    let server_config = state.server_config.clone();
                    return Task::future(async move {
                        match config_to_save.save().await {
                            Ok(_) => {}
                            Err(e) => {
                                Message::PostToast(Toast {
                                    title: "Error saving settings".into(),
                                    body: format!("{}", e),
                                    ty: ToastType::Error,
                                });
                            }
                        }

                        let mut config = server_config.write().await;
                        *config = config_to_save.clone();

                        info!("Updated server config");
                        SettingsMessage::SavedConfig(config_to_save).into()
                    });
                }
                SettingsMessage::SavedConfig(c) => {
                    state.config = c;
                    v.dirty = false;
                }
            }
        }
        Task::none()
    }
}
