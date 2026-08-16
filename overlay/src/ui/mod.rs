mod avatar;
mod participant_list;
mod speaking_indicator;

pub use avatar::AvatarWidget;
pub use participant_list::ParticipantList;
pub use speaking_indicator::SpeakingIndicator;

use gtk4::prelude::*;
use gtk4::{Box, Orientation, ScrolledWindow, PolicyType};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::config::Config;
use crate::protocol::Snapshot;
use crate::lifecycle::OverlayCommand;

pub struct OverlayUI {
    container: Box,
    participant_list: ParticipantList,
    _scrolled: ScrolledWindow,
    ui_tx: mpsc::UnboundedSender<OverlayCommand>,
}

impl OverlayUI {
    pub async fn new(window: &gtk4::ApplicationWindow, config: &Config, ui_tx: mpsc::UnboundedSender<OverlayCommand>) -> anyhow::Result<Arc<Self>> {
        // Main container
        let container = Box::new(Orientation::Vertical, 8);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        container.set_margin_start(12);
        container.set_margin_end(12);
        container.add_css_class("overlay-container");

        // Scrolled window for participant list
        let scrolled = ScrolledWindow::new();
        scrolled.set_policy(PolicyType::Never, PolicyType::Automatic);
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(false);
        scrolled.add_css_class("overlay-scrolled");

        let participant_list = ParticipantList::new(config)?;
        scrolled.set_child(Some(participant_list.widget()));
        
        container.append(&scrolled);

        // Apply CSS
        Self::apply_css();

        let ui = Arc::new(Self {
            container,
            participant_list,
            _scrolled: scrolled,
            ui_tx,
        });

        window.set_child(Some(&ui.container));
        
        Ok(ui)
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }

    pub fn update_from_snapshot(&self, snapshot: &Snapshot) {
        self.participant_list.update(snapshot);
    }

    fn apply_css() {
        let css = include_str!("style.css");
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(css);
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("No display"),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
