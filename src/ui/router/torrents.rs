use std::time::Duration;

use anawt::{AnawtTorrentStatus, InfoHash, RemoveFlags, TorrentState};
use freya::{
    prelude::*,
    query::{Mutation, Query, QueryStateData, use_mutation, use_query},
    sdk::use_track_watcher,
};
use tokio::sync::watch;

use crate::ui::{
    DEFAULT_CORNER_RADIUS, DEFAULT_PAGE_PADDING,
    components::Spacer,
    icons,
    queries::{FetchTorrentWatchers, RemoveTorrent},
};

#[derive(PartialEq)]
pub struct Torrents;

impl Component for Torrents {
    fn render(&self) -> impl IntoElement {
        let watchers_query = use_query(Query::new((), FetchTorrentWatchers));

        let torrent_list = match &*watchers_query.read().state() {
            QueryStateData::Settled {
                res: Ok(watchers), ..
            } => {
                let children = watchers
                    .iter()
                    .map(|w| TorrentEntry::new(w.clone()).into_element())
                    .collect::<Vec<_>>();
                rect().vertical().children(children).into_element()
            }
            QueryStateData::Settled { res: Err(e), .. } => {
                rect().child(label().text(e.to_string())).into_element()
            }
            _ => CircularLoader::new().into_element(),
        };

        rect().child(torrent_list).padding(DEFAULT_PAGE_PADDING)
    }
}

pub struct TorrentEntry {
    watcher: watch::Receiver<AnawtTorrentStatus>,
}

impl TorrentEntry {
    pub fn new(watcher: watch::Receiver<AnawtTorrentStatus>) -> Self {
        Self { watcher }
    }
}

impl PartialEq for TorrentEntry {
    fn eq(&self, other: &Self) -> bool {
        // self.watcher.same_channel(&other.watcher)
        true
    }
}

fn format_bytes(bytes: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    let float_bytes = bytes as f64;

    let unit_index = (float_bytes.log2() / 10.0).floor() as usize;
    let unit_index = unit_index.min(UNITS.len() - 1);

    let divisor = 1024.0f64.powf(unit_index as f64);

    let value = float_bytes / divisor;

    if unit_index > 2 {
        format!("{:.2}{}", value, UNITS[unit_index])
    } else {
        format!("{:.0}{}", value, UNITS[unit_index])
    }
}

impl Component for TorrentEntry {
    fn render(&self) -> impl IntoElement {
        use_track_watcher(&self.watcher);
        let remove_mutation = use_mutation(Mutation::new(RemoveTorrent));
        let status = self.watcher.borrow().clone();

        let torrent_context = Menu::new().child(
            MenuButton::new()
                .child("RemoveTorrent")
                .on_press(move |_| remove_mutation.mutate((status.info_hash, RemoveFlags::all()))),
        );

        let extra_elements = vec![
            label()
                .text(format_bytes(status.total_bytes))
                .font_size(11.)
                .color(Color::DARK_GRAY)
                .into_element(),
            TooltipContainer::new(Tooltip::new(status.save_path.clone()))
                .child(
                    label()
                        .text(status.save_path)
                        .text_overflow(TextOverflow::Ellipsis)
                        .width(Size::px(200.))
                        .font_size(11.)
                        .color(Color::DARK_GRAY),
                )
                .position(AttachedPosition::Bottom)
                .into_element(),
        ];
        let rem = if status.state == TorrentState::Downloading {
            let remaining_bytes = status.total_bytes as f64 * (1.0 - status.progress);
            let remaining_seconds = remaining_bytes / status.download_rate as f64;

            let duration = Duration::from_secs_f64(remaining_seconds);

            let seconds = duration.as_secs() % 60;
            let minutes = (duration.as_secs() / 60) % 60;
            let hours = (duration.as_secs() / 60) / 60;

            let duration_str = format!("{}:{}:{}", hours, minutes, seconds);
            label().text(duration_str).into_element()
        } else {
            rect().into_element()
        };

        rect()
            .background(Color::LIGHT_GRAY)
            .padding(10.)
            .spacing(5.)
            .corner_radius(DEFAULT_CORNER_RADIUS)
            .vertical()
            .child(
                rect()
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .content(Content::Flex)
                    .child(status.name)
                    .child(Spacer::horizontal_fill())
                    .child(rem)
                    .child(
                        rect()
                            .horizontal()
                            .center()
                            .child(svg(icons::ARROW_CIRCLE_DOWN_ICON).width(Size::px(14.)))
                            .child(
                                label()
                                    .text(format!(
                                        "{}/s",
                                        format_bytes(status.download_rate as i64)
                                    ))
                                    .font_size(14.),
                            ),
                    )
                    .child(
                        rect()
                            .horizontal()
                            .center()
                            .child(svg(icons::ARROW_CIRCLE_UP_ICON).width(Size::px(14.)))
                            .child(
                                label()
                                    .text(format!("{}/s", format_bytes(status.upload_rate as i64)))
                                    .font_size(14.),
                            ),
                    ),
            )
            .child(
                rect()
                    .horizontal()
                    .children(extra_elements)
                    .spacing(15.)
                    .cross_align(Alignment::Center),
            )
            .child(
                ProgressBar::new(status.progress as f32 * 100.)
                    .show_progress(false)
                    .height(5.),
            )
            .on_secondary_down(move |_| {
                ContextMenu::open(torrent_context.clone());
            })
            .width(Size::Fill)
    }
}
