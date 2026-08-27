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

        Self::apply_css(window);

        let ui = Rc::new(Self {
            container,
            participant_list,
            config,
            last_snapshot: RefCell::new(None),
            _scrolled: scrolled,
        });

        // The scrolled window's min-content sizes give the empty panel a
        // 236x58 natural box, which would render as an opaque rectangle on
        // the desktop before the first snapshot arrives. The container only
        // becomes visible when a snapshot yields visible participant rows.
        ui.container.set_visible(false);

        window.set_child(Some(&ui.container));

        Ok(ui)
    }

    pub fn update_from_snapshot(&self, snapshot: &Snapshot) -> bool {
        *self.last_snapshot.borrow_mut() = Some(snapshot.clone());
        let visible = self.participant_list.update(snapshot);
        self.container.set_visible(visible);
        visible
    }

    pub fn update_settings(&self, settings: OverlaySettings) -> bool {
        self.config.borrow_mut().apply_overlay_settings(settings);
        // No reset: keyed participant rows are updated in place so settings
        // changes never recreate widgets or re-download avatars. The
        // enabled=false path inside update() still clears rows.
        let visible = self
            .last_snapshot
            .borrow()
            .as_ref()
            .is_some_and(|snapshot| self.participant_list.update(snapshot));
        self.container.set_visible(visible);
        visible
    }

    fn apply_css(window: &gtk4::ApplicationWindow) {
        let css = include_str!("style.css");
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(css);
        // Display-level provider (covers all widgets)
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("No display"),
            &provider,
            800, // above Adwaita's APPLICATION priority (600)
        );
        // Window-level provider: ensures the window node itself is themed
        window
            .style_context()
            .add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{ParticipantSelf, PROTOCOL_VERSION};

    fn snapshot_with_speaking_self(speaking: bool) -> Snapshot {
        Snapshot {
            version: PROTOCOL_VERSION,
            timestamp: 0,
            self_: ParticipantSelf {
                user_id: "me".to_string(),
                username: "Me".to_string(),
                avatar_url: String::new(),
                mute: false,
                deaf: false,
                speaking,
            },
            participants: Vec::new(),
        }
    }

    /// Regression guard for the empty-panel rectangle: the scrolled window's
    /// min-content sizes (220x42) give the empty `.overlay-container` a
    /// natural box, so a visible container at startup rendered as an opaque
    /// ~236x58 rectangle on the desktop before Vesktop ever connected. The
    /// container must stay hidden until a snapshot yields visible rows and
    /// hide again whenever no participant is visible.
    #[test]
    fn container_visibility_tracks_visible_participants() {
        if gtk4::init().is_err() {
            eprintln!("skipping: no display available for GTK widget test");
            return;
        }

        // Plain unassociated window (no GApplication): creating an
        // application-owned window before ::startup emits a Gtk-CRITICAL.
        let window = gtk4::ApplicationWindow::builder().build();
        // is_visible() accounts for ancestors: mark the window visible so the
        // assertions reflect the container's own visibility flag.
        window.present();
        let ui = OverlayUI::new(&window, &Config::default()).expect("ui builds");

        assert!(
            !ui.container.is_visible(),
            "empty overlay must not render before the first snapshot"
        );

        // Default user_display is speaking_only: a speaking self is visible.
        assert!(ui.update_from_snapshot(&snapshot_with_speaking_self(true)));
        assert!(ui.container.is_visible());

        assert!(!ui.update_from_snapshot(&snapshot_with_speaking_self(false)));
        assert!(
            !ui.container.is_visible(),
            "container must hide when no participant is visible"
        );
    }
}
