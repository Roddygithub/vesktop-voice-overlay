use anyhow::Result;
use gtk4::prelude::*;
use gtk4::{Box, Label, ListBox, ListBoxRow, Orientation, SelectionMode, Align, pango};
use std::sync::Arc;

use crate::config::Config;
use crate::protocol::{Participant, Snapshot};
use crate::ui::{AvatarWidget, SpeakingIndicator};

pub struct ParticipantList {
    list_box: ListBox,
    config: Arc<Config>,
}

impl ParticipantList {
    pub fn new(config: &Config) -> Result<Self> {
        let list_box = ListBox::new();
        list_box.set_selection_mode(SelectionMode::None);
        list_box.add_css_class("participant-list");

        let config = Arc::new(config.clone());
        Ok(Self { list_box, config })
    }

    pub fn widget(&self) -> &ListBox {
        &self.list_box
    }

    pub fn update(&self, snapshot: &Snapshot) {
        // Clear existing rows
        while let Some(row) = self.list_box.first_child() {
            self.list_box.remove(&row);
        }

        // Add self first
        let self_row = Self::create_participant_row(&self.config, &self.into_participant(&snapshot.self_), true);
        self.list_box.append(&self_row);

        // Add other participants (limit to max_participants)
        let max = self.config.overlay.max_participants;
        for (i, participant) in snapshot.participants.iter().enumerate() {
            if i >= max { break; }
            let row = Self::create_participant_row(&self.config, participant, false);
            self.list_box.append(&row);
        }
    }

    fn into_participant(&self, self_: &crate::protocol::ParticipantSelf) -> Participant {
        Participant {
            user_id: self_.user_id.clone(),
            username: self_.username.clone(),
            avatar_url: self_.avatar_url.clone(),
            speaking: self_.speaking,
            volume: None,
        }
    }

    fn create_participant_row(config: &Config, participant: &Participant, is_self: bool) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.add_css_class("participant-row");
        row.set_activatable(false);

        let hbox = Box::new(Orientation::Horizontal, 0);
        hbox.set_margin_top(4);
        hbox.set_margin_bottom(4);
        hbox.set_margin_start(8);
        hbox.set_margin_end(8);

        // Avatar
        let avatar = AvatarWidget::new(&participant.avatar_url, config.overlay.avatar_size);
        hbox.append(avatar.widget());

        // Name
        let name_label = Label::new(Some(&participant.username));
        name_label.add_css_class("participant-name");
        name_label.set_halign(Align::Start);
        name_label.set_ellipsize(pango::EllipsizeMode::End);
        name_label.set_hexpand(true);
        hbox.append(&name_label);

        // Speaking indicator
        let speaking_indicator = SpeakingIndicator::new(participant.speaking);
        hbox.append(speaking_indicator.widget());

        // Self indicator
        if is_self {
            let self_indicator = Label::new(Some("●"));
            self_indicator.add_css_class("self-indicator");
            self_indicator.set_tooltip_text(Some("You"));
            hbox.append(&self_indicator);
        }

        // Volume (if available)
        if let Some(volume) = participant.volume {
            let volume_label = Label::new(Some(&format!("{}%", volume)));
            volume_label.add_css_class("volume-indicator");
            hbox.append(&volume_label);
        }

        row.set_child(Some(&hbox));
        row
    }
}
