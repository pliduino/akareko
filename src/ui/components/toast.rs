use futures::SinkExt;
use iced::{
    Element, stream,
    widget::{button, column, text},
};
use tokio::sync::mpsc;
use tracing::error;

use crate::{db::Repositories, ui::Message};

#[derive(Debug, Clone)]
pub struct Toast {
    pub title: String,
    pub body: String,
    pub ty: ToastType,
}

#[derive(Debug, Clone)]
pub enum ToastType {
    Info,
    Error,
}

impl Toast {
    pub fn view(&self, index: usize) -> Element<Message> {
        column![
            button(text("X")).on_press(Message::CloseToast(index)),
            text(&self.title),
            text(&self.body)
        ]
        .into()
    }
}

pub fn toast_worker() -> impl iced::futures::Stream<Item = Message> {
    stream::channel(100, |mut output| async move {
        let (tx, mut rx) = mpsc::channel::<Toast>(100);
        match output.send(Message::ToastSenderReady(tx)).await {
            Ok(()) => {}
            Err(e) => {
                // This should honestly never happen, it's here just in case
                error!("Error initializing toast subscriptions: {}", e);
            }
        };

        loop {
            let toast = match rx.recv().await {
                Some(toast) => toast,
                None => break,
            };

            match output.send(Message::PostToast(toast)).await {
                Ok(()) => {}
                Err(e) => {
                    if e.is_disconnected() {
                        error!("Disconnected from toast output");
                    } else if e.is_full() {
                        error!("Toast output is full");
                    }
                }
            };
        }
    })
}
