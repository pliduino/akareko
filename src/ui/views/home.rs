use iced::{
    Subscription, Task,
    widget::{button, column, text},
};

use crate::{
    db::Repositories,
    ui::{
        AppState, Message,
        components::modal::{Modal, add_who::AddWhoModal},
        views::{NovelListView, View, settings::SettingsView, user_list::UserListView},
    },
};

#[derive(Debug, Clone)]
pub struct HomeView {}

#[derive(Debug, Clone)]
pub enum HomeMessage {}

impl HomeView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        Subscription::none()
    }

    pub fn on_enter(state: &mut AppState) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self, state: &AppState) -> iced::Element<Message> {
        column![
            button(text("Novels"))
                .on_press(Message::ChangeView(View::NovelList(NovelListView::new()))),
            button(text("Settings"))
                .on_press(Message::ChangeView(View::Settings(SettingsView::new()))),
            button(text("Add user"))
                .on_press(Message::OpenModal(Modal::AddWho(AddWhoModal::new()))),
            button(text("SaveTorrent")).on_press(Message::SaveTorrent),
            button(text("User List"))
                .on_press(Message::ChangeView(View::UserList(UserListView::new()))),
        ]
        .into()
    }

    pub fn update(m: HomeMessage, state: &mut AppState) -> Task<Message> {
        Task::none()
    }
}
