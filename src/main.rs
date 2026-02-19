use std::rc::Rc;

use iced::Task;
use iced::Theme;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::config::AuroraConfig;
use crate::db::Repositories;

use crate::ui::AppState;
use crate::ui::Message;

mod config;
mod db;
mod errors;
mod hash;
mod helpers;
mod models;
mod server;
mod ui;

#[cfg(feature = "sqlite")]
mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations/sqlite");
}

fn main() -> Result<(), ()> {
    let format = time::format_description::parse("[hour]:[minute]:[second]").expect("Cataplum");

    let timer = fmt::time::LocalTime::new(format);

    let filter = EnvFilter::builder().parse_lossy("none,aurora=info,anawt=info");

    let stdout_log = fmt::layer()
        .compact()
        .with_line_number(false)
        .with_target(false)
        .with_timer(timer)
        .with_filter(filter);
    tracing_subscriber::registry().with(stdout_log).init();

    info!("Initializing Application...");

    iced::daemon(
        || {
            (
                AppState::new(),
                Task::done(Message::OpenWindow).chain(Task::perform(AuroraConfig::load(), |c| {
                    Message::ConfigLoaded(c)
                })),
            )
        },
        AppState::update,
        AppState::view,
    )
    .subscription(AppState::subscription)
    .theme(|s: &AppState, _| s.theme())
    .run()
    .unwrap();

    Ok(())
}
