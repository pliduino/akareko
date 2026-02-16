use iced::{
    Task,
    widget::{Column, button, center, column, row, text, text_input},
};

use crate::{
    db::{
        Content, ContentEntry, Index, Magnet,
        index::{NovelChapter, NovelTag},
    },
    helpers::{Language, now_timestamp},
    ui::{
        AppState, Message,
        views::{View, ViewMessage, novel::NovelView},
    },
};

#[derive(Debug, Clone, Default)]
struct ContentEntryValues {
    title: String,
    path: String,
    enumeration: f32,
}

#[derive(Debug, Clone)]
pub struct AddNovelChapterView {
    novel: Index<NovelTag>,
    magnet: String,
    entries: Vec<ContentEntryValues>,
}

#[derive(Debug, Clone)]
pub enum AddNovelChapterMessage {
    AddContent,

    UpdateTitle(String, usize),
    UpdateEnumeration(f32, usize),
    UpdatePath(String, usize),
    AddEntry,
    RemoveEntry(usize),

    UpdateMagnet(String),
    SavedContent,
}

impl From<AddNovelChapterMessage> for Message {
    fn from(m: AddNovelChapterMessage) -> Self {
        Message::ViewMessage(ViewMessage::AddChapter(m))
    }
}

impl AddNovelChapterView {
    pub fn new(novel: Index<NovelTag>) -> Self {
        Self {
            novel,
            magnet: String::new(),
            entries: vec![],
        }
    }

    pub fn on_enter(state: &mut AppState) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self, state: &AppState) -> iced::Element<Message> {
        let entries = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                column![
                    text_input("Title", &e.title)
                        .on_input(move |s| AddNovelChapterMessage::UpdateTitle(s, i).into())
                        .width(iced::Length::Fill),
                    text_input("Path", &e.path)
                        .on_input(move |s| AddNovelChapterMessage::UpdatePath(s, i).into())
                        .width(iced::Length::Fill)
                ]
                .into()
            })
            .collect();

        let entries_column = Column::from_vec(entries).width(iced::Length::Fill);

        column![
            text_input("Magnet", &self.magnet)
                .on_input(|s| AddNovelChapterMessage::UpdateMagnet(s).into()),
            center(row![
                button(text("+")).on_press(AddNovelChapterMessage::AddEntry.into()),
                button(text("-")).on_press_maybe(match self.entries.len() {
                    0 => None,
                    _ => Some(AddNovelChapterMessage::RemoveEntry(self.entries.len() - 1).into()),
                }),
            ],)
            .height(iced::Length::Shrink),
            entries_column,
            button(text("Add Chapter")).on_press(AddNovelChapterMessage::AddContent.into())
        ]
        .into()
    }

    pub fn update(m: AddNovelChapterMessage, state: &mut AppState) -> Task<Message> {
        if let View::AddChapter(v) = &mut state.view {
            match m {
                AddNovelChapterMessage::AddContent => {
                    if let Some(repositories) = &state.repositories {
                        let index_hash = v.novel.hash().clone();

                        let entries: Vec<ContentEntry<NovelTag>> = v
                            .entries
                            .iter()
                            .map(|e| ContentEntry {
                                title: e.title.clone(),
                                enumeration: e.enumeration,
                                path: e.path.clone(),
                                content: NovelChapter::new(Language::English),
                                progress: 0.0,
                            })
                            .collect();

                        let chapter = Content::new_signed(
                            state.config.public_key().clone(),
                            index_hash,
                            now_timestamp(),
                            Magnet(v.magnet.clone()),
                            entries,
                            state.config.private_key(),
                        );

                        let repositories = repositories.clone();
                        return Task::future(async move {
                            match repositories.index().await.add_content(chapter).await {
                                Ok(_) => {}
                                Err(e) => {
                                    println!("Error adding chapter: {}", e);
                                }
                            }
                            AddNovelChapterMessage::SavedContent.into()
                        });
                    }
                }
                AddNovelChapterMessage::UpdateTitle(title, i) => {
                    v.entries[i].title = title;
                }
                AddNovelChapterMessage::UpdateEnumeration(enumeration, i) => {
                    v.entries[i].enumeration = enumeration;
                }
                AddNovelChapterMessage::UpdatePath(path, i) => {
                    v.entries[i].path = path;
                }
                AddNovelChapterMessage::UpdateMagnet(magnet) => {
                    v.magnet = magnet;
                }
                AddNovelChapterMessage::AddEntry => {
                    v.entries.push(ContentEntryValues::default());
                }
                AddNovelChapterMessage::RemoveEntry(i) => {
                    v.entries.remove(i);
                }
                AddNovelChapterMessage::SavedContent => {
                    v.entries = vec![];
                    v.magnet = String::new();
                    return Task::done(Message::ChangeView(View::Novel(NovelView::new(
                        v.novel.clone(),
                    ))));
                }
            }
        }
        Task::none()
    }
}
