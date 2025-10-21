// use iced::widget::{button, row, text};

// use crate::{
//     db::{Repositories, novel::NovelChapter},
//     ui::Message,
// };

// pub struct NovelChapterComponent {
//     chapter: NovelChapter,
// }

// impl NovelChapterComponent {
//     pub fn new(content: NovelChapter) -> Self {
//         Self { chapter: content }
//     }

//     pub fn view<R: Repositories + 'static>(&self) -> iced::Element<Message<R>> {
//         row![
//             text(&self.chapter.enumeration),
//             text(&self.chapter.title),
//             button(text("Download")).on_press(Message::DownloadTorrent{
//                 self.chapter.content.magnet_link.clone()

//             }
//             ),
//         ]
//         .into()
//     }
// }
