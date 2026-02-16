pub mod add_chapter;
pub mod add_novel;
pub mod home;
pub mod image_viewer;
pub mod novel;
pub mod novel_list;
pub mod post;
pub mod settings;
pub mod user_list;

use iced::Task;

use crate::ui::{
    AppState, Message,
    views::{
        add_chapter::{AddNovelChapterMessage, AddNovelChapterView},
        add_novel::{AddNovelMessage, AddNovelView},
        home::{HomeMessage, HomeView},
        image_viewer::{ImageViewerMessage, ImageViewerView},
        novel::{NovelMessage, NovelView},
        novel_list::{NovelListMessage, NovelListView},
        post::{PostMessage, PostView},
        settings::{SettingsMessage, SettingsView},
        user_list::{UserListMessage, UserListView},
    },
};

#[derive(Debug, Clone)]
pub enum View {
    Home(HomeView),
    NovelList(NovelListView),
    Novel(NovelView),
    AddNovel(AddNovelView),
    AddChapter(AddNovelChapterView),
    Settings(SettingsView),
    ImageViewer(ImageViewerView),
    UserList(UserListView),
    Post(PostView),
}

#[derive(Debug, Clone)]
pub enum ViewMessage {
    Home(HomeMessage),
    NovelList(NovelListMessage),
    Novel(NovelMessage),
    AddNovel(AddNovelMessage),
    AddChapter(AddNovelChapterMessage),
    Settings(SettingsMessage),
    ImageViewer(ImageViewerMessage),
    UserList(UserListMessage),
    Post(PostMessage),
}

impl View {
    pub fn on_enter(state: &mut AppState) -> Task<Message> {
        match state.view {
            View::Home(_) => HomeView::on_enter(state),
            View::NovelList(_) => NovelListView::on_enter(state),
            View::Novel(_) => NovelView::on_enter(state),
            View::AddNovel(_) => AddNovelView::on_enter(state),
            View::AddChapter(_) => AddNovelChapterView::on_enter(state),
            View::Settings(_) => SettingsView::on_enter(state),
            View::ImageViewer(_) => ImageViewerView::on_enter(state),
            View::UserList(_) => UserListView::on_enter(state),
            View::Post(_) => PostView::on_enter(state),
        }
    }

    pub fn view(state: &AppState) -> iced::Element<'_, Message> {
        match &state.view {
            View::Home(v) => v.view(state),
            View::NovelList(v) => v.view(state),
            View::Novel(v) => v.view(state),
            View::AddNovel(v) => v.view(state),
            View::AddChapter(v) => v.view(state),
            View::Settings(v) => v.view(state),
            View::ImageViewer(v) => v.view(state),
            View::UserList(v) => v.view(state),
            View::Post(v) => v.view(state),
        }
    }

    pub fn update(message: ViewMessage, state: &mut AppState) -> Task<Message> {
        match message {
            ViewMessage::Home(m) => HomeView::update(m, state),
            ViewMessage::NovelList(m) => NovelListView::update(m, state),
            ViewMessage::Novel(m) => NovelView::update(m, state),
            ViewMessage::AddNovel(m) => AddNovelView::update(m, state),
            ViewMessage::AddChapter(m) => AddNovelChapterView::update(m, state),
            ViewMessage::Settings(m) => SettingsView::update(m, state),
            ViewMessage::ImageViewer(m) => ImageViewerView::update(m, state),
            ViewMessage::UserList(m) => UserListView::update(m, state),
            ViewMessage::Post(m) => PostView::update(m, state),
        }
    }
}
