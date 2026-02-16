use iced::{
    Task,
    widget::{Button, Column, button, column, row, text},
};

use crate::{
    db::{
        Index, Repositories,
        index::{IndexRepository, NovelTag},
    },
    ui::{
        AppState, Message,
        views::{View, ViewMessage, add_novel::AddNovelView, novel::NovelView},
    },
};

#[derive(Debug, Clone)]
pub struct NovelListView {
    novels: Vec<Index<NovelTag>>,
}

#[derive(Debug, Clone)]
pub enum NovelListMessage {
    LoadedNovels(Vec<Index<NovelTag>>),
}

impl From<NovelListMessage> for Message {
    fn from(msg: NovelListMessage) -> Message {
        Message::ViewMessage(ViewMessage::NovelList(msg))
    }
}

impl NovelListView {
    pub fn new() -> Self {
        Self { novels: vec![] }
    }

    pub fn on_enter(state: &mut AppState) -> Task<Message> {
        if let Some(repositories) = &state.repositories {
            let repositories = repositories.clone();

            return Task::future(async move {
                let novels = repositories.index().await.get_indexes().await;
                NovelListMessage::LoadedNovels(novels).into()
            });
        }
        Task::none()
    }

    pub fn view(&self, state: &AppState) -> iced::Element<'_, Message> {
        let mut column: Vec<iced::Element<Message>> = vec![text("Novels").into()];

        if state.config.dev_mode() {
            column.push(
                button(text("Add Novel"))
                    .on_press(Message::ChangeView(View::AddNovel(AddNovelView::new())))
                    .into(),
            );
        }

        for novel in self.novels.iter() {
            column.push(
                button(text(novel.title().clone()))
                    .on_press(Message::ChangeView(View::Novel(NovelView::new(
                        novel.clone(),
                    ))))
                    .into(),
            );
        }

        Column::from_vec(column).into()
    }

    pub fn update(m: NovelListMessage, state: &mut AppState) -> Task<Message> {
        if let View::NovelList(v) = &mut state.view {
            match m {
                NovelListMessage::LoadedNovels(novels) => {
                    v.novels = novels;
                }
            }
        }
        Task::none()
    }
}
