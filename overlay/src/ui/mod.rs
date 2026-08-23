mod avatar;
mod participant_list;

pub use avatar::AvatarWidget;
pub use participant_list::ParticipantList;

use gtk4::prelude::*;
use gtk4::{Box, Orientation, PolicyType, ScrolledWindow};
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{Config, OverlaySettings};
use crate::protocol::Snapshot;

pub struct OverlayUI {
    container: Box,
    participant_list: ParticipantList,
    config: Rc<RefCell<Config>>,
    last_snapshot: RefCell<Option<Snapshot>>,
    _scrolled: ScrolledWindow,
}

impl OverlayUI {
    pub fn new(window: &gtk4::ApplicationWindow, config: &Config) -> anyhow::Result<Rc<Self>> {
        let container = Box::new(Orientation::Vertical, 0);
        container.add_css_class("overlay-container");

        let scrolled = ScrolledWindow::new();
        scrolled.set_policy(PolicyType::Never, PolicyType::Automatic);
        scrolled.set_vexpand(false);
        scrolled.set_hexpand(false);
        scrolled.set_min_content_width(220);
        scrolled.set_min_content_height(42);
        scrolled.set_propagate_natural_height(true);
        scrolled.add_css_class("overlay-scrolled");

        let config = Rc::new(RefCell::new(config.clone()));
        let participant_list = ParticipantList::new(config.clone())?;
        scrolled.set_child(Some(participant_list.widget()));

        container.append(&scrolled);

        Self::apply_css();

        let ui = Rc::new(Self {
            container,
            participant_list,
            config,
            last_snapshot: RefCell::new(None),
            _scrolled: scrolled,
        });

        window.set_child(Some(&ui.container));

        Ok(ui)
    }

    pub fn update_from_snapshot(&self, snapshot: &Snapshot) -> bool {
        *self.last_snapshot.borrow_mut() = Some(snapshot.clone());
        self.participant_list.update(snapshot)
    }

    pub fn update_settings(&self, settings: OverlaySettings) -> bool {
        self.config.borrow_mut().apply_overlay_settings(settings);
        // No reset: keyed participant rows are updated in place so settings
        // changes never recreate widgets or re-download avatars. The
        // enabled=false path inside update() still clears rows.
        self.last_snapshot
            .borrow()
            .as_ref()
            .is_some_and(|snapshot| self.participant_list.update(snapshot))
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
