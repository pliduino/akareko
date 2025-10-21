use std::{fs::File, io::Read, path::PathBuf};

use bytes::Bytes;
use iced::{
    Task,
    widget::{
        self, Column, Scrollable, button,
        canvas::Image,
        center, column,
        image::{self, Handle},
        row, text, text_input,
    },
};
use tracing::info;
use zip::ZipArchive;

use crate::{
    db::{Index, Repositories, index::IndexRepository},
    ui::{
        AppState, Message,
        views::{View, ViewMessage, novel_list::NovelListView},
    },
};

#[derive(Debug, Clone)]
pub struct ImageViewerView {
    file_path: PathBuf,
    images: Vec<Image>,

    // Starts at 1 and go up to len, use -1 to get index
    cur_page: usize,
}

#[derive(Debug, Clone)]
pub enum ImageViewerMessage {
    LoadedImages(Vec<Image>),
    PrevPage,
    NextPage,
}

impl From<ImageViewerMessage> for Message {
    fn from(m: ImageViewerMessage) -> Self {
        Message::ViewMessage(ViewMessage::ImageViewer(m))
    }
}

impl ImageViewerView {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            images: vec![],
            cur_page: 1,
        }
    }

    pub fn on_enter(state: &mut AppState) -> Task<Message> {
        if let View::ImageViewer(v) = &mut state.view {
            let path = v.file_path.clone();
            return Task::future(async move {
                if let Some(extension) = path.extension() {
                    if extension == "cbz" {
                        let file = File::open(path).unwrap();
                        let mut zip = ZipArchive::new(file).unwrap();
                        let mut images = vec![];
                        for i in 0..zip.len() {
                            let mut f = zip.by_index(i).unwrap();
                            let mut buffer = vec![];
                            f.read_to_end(&mut buffer).unwrap();
                            let bytes = Bytes::from(buffer);
                            images.push(Image::new(Handle::from_bytes(bytes)));
                        }
                        return ImageViewerMessage::LoadedImages(images).into();
                    }
                }

                ImageViewerMessage::LoadedImages(vec![]).into()
            });
        }

        Task::none()
    }

    pub fn view(&self, state: &AppState) -> iced::Element<Message> {
        let image_area = if self.images.len() > 0 {
            Scrollable::new(
                center(widget::image(self.images[self.cur_page - 1].handle.clone()))
                    .center_y(iced::Length::Shrink),
            )
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
        } else {
            Scrollable::new(text("Loading..."))
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
        };

        column![
            row![
                button(text("<")).on_press_maybe(if self.cur_page <= 1 {
                    None
                } else {
                    Some(ImageViewerMessage::PrevPage.into())
                }),
                text(format!("{} / {}", self.cur_page, self.images.len())),
                button(text(">")).on_press_maybe(if self.cur_page >= self.images.len() {
                    None
                } else {
                    Some(ImageViewerMessage::NextPage.into())
                }),
            ],
            image_area
        ]
        .align_x(iced::alignment::Horizontal::Center)
        .width(iced::Length::Fill)
        .into()
    }

    pub fn update(m: ImageViewerMessage, state: &mut AppState) -> Task<Message> {
        if let View::ImageViewer(v) = &mut state.view {
            match m {
                ImageViewerMessage::LoadedImages(images) => {
                    v.images = images;
                    if v.cur_page > v.images.len() {
                        v.cur_page = v.images.len();
                    }
                }
                ImageViewerMessage::PrevPage => {
                    if v.cur_page > 1 {
                        v.cur_page -= 1;
                    }
                }
                ImageViewerMessage::NextPage => {
                    if v.cur_page < v.images.len() {
                        v.cur_page += 1;
                    }
                }
            }
        }
        Task::none()
    }
}
