use anyhow::Result;
use gtk4::prelude::*;
use gtk4::{pango, Align, Box, Image, Label, ListBox, ListBoxRow, Orientation, SelectionMode};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::config::{Config, NameDisplayMode, UserDisplayMode};
use crate::protocol::{Participant, Snapshot};
use crate::ui::AvatarWidget;

pub struct ParticipantList {
    list_box: ListBox,
    config: Rc<RefCell<Config>>,
    rows: RefCell<HashMap<String, ParticipantRow>>,
}

struct ParticipantRow {
    row: ListBoxRow,
    name: Label,
    avatar: Rc<AvatarWidget>,
    mute_icon: Image,
    deaf_icon: Image,
}

impl ParticipantList {
    pub fn new(config: Rc<RefCell<Config>>) -> Result<Self> {
        let list_box = ListBox::new();
        list_box.set_selection_mode(SelectionMode::None);
        list_box.add_css_class("participant-list");

        Ok(Self {
            list_box,
            config,
            rows: RefCell::new(HashMap::new()),
        })
    }

    pub fn widget(&self) -> &ListBox {
        &self.list_box
    }

    pub fn update(&self, snapshot: &Snapshot) -> bool {
        let config = self.config.borrow();
        if !config.overlay.enabled {
            drop(config);
            self.reset();
            return false;
        }

        let user_display = config.overlay.user_display;
        let name_display = config.overlay.name_display;
        let max = config.overlay.max_participants;
        let mut desired = Vec::with_capacity(snapshot.participants.len() + 1);
        let self_participant = self.to_participant(&snapshot.self_);
        if Self::is_user_visible(user_display, &self_participant) {
            desired.push((snapshot.self_.user_id.clone(), self_participant));
        }

        desired.extend(
            snapshot
                .participants
                .iter()
                .filter(|participant| Self::is_user_visible(user_display, participant))
                .take(max)
                .map(|participant| (participant.user_id.clone(), participant.clone())),
        );

        let desired_ids: HashSet<_> = desired.iter().map(|(id, _)| id.clone()).collect();
        let mut rows = self.rows.borrow_mut();
        for id in rows
            .keys()
            .filter(|id| !desired_ids.contains(*id))
            .cloned()
            .collect::<Vec<_>>()
        {
            if let Some(participant_row) = rows.remove(&id) {
                self.list_box.remove(&participant_row.row);
            }
        }

        for (user_id, participant) in desired {
            if let Some(existing) = rows.get(&user_id) {
                existing.update(
                    &participant,
                    Self::is_name_visible(name_display, &participant),
                    config.avatar_size_px(),
                );
            } else {
                let participant_row = ParticipantRow::new(&config, &participant);
                self.list_box.append(&participant_row.row);
                rows.insert(user_id, participant_row);
            }
        }

        !desired_ids.is_empty()
    }

    pub fn reset(&self) {
        let mut rows = self.rows.borrow_mut();
        for (_, participant_row) in rows.drain() {
            self.list_box.remove(&participant_row.row);
        }
    }

    fn to_participant(&self, self_: &crate::protocol::ParticipantSelf) -> Participant {
        Participant {
            user_id: self_.user_id.clone(),
            username: self_.username.clone(),
            avatar_url: self_.avatar_url.clone(),
            mute: self_.mute,
            deaf: self_.deaf,
            speaking: self_.speaking,
            volume: None,
        }
    }

    fn is_user_visible(user_display: UserDisplayMode, participant: &Participant) -> bool {
        match user_display {
            UserDisplayMode::Always => true,
            UserDisplayMode::SpeakingOnly => participant.speaking,
        }
    }

    fn is_name_visible(name_display: NameDisplayMode, participant: &Participant) -> bool {
        match name_display {
            NameDisplayMode::Always => true,
            NameDisplayMode::SpeakingOnly => participant.speaking,
            NameDisplayMode::Never => false,
        }
    }
}

impl ParticipantRow {
    fn new(config: &Config, participant: &Participant) -> Self {
        let row = ListBoxRow::new();
        row.add_css_class("participant-row");
        row.set_activatable(false);

        let hbox = Box::new(Orientation::Horizontal, 4);
        hbox.set_margin_top(2);
        hbox.set_margin_bottom(2);
        hbox.set_margin_start(2);
        hbox.set_margin_end(2);

        let avatar_size = config.avatar_size_px();
        let avatar = AvatarWidget::new(&participant.avatar_url, avatar_size);
        hbox.append(avatar.widget());

        let name_label = Label::new(Some(&participant.username));
        name_label.add_css_class("participant-name");
        name_label.set_halign(Align::Start);
        name_label.set_ellipsize(pango::EllipsizeMode::End);
        name_label.set_hexpand(true);
        hbox.append(&name_label);

        let mute_icon = Image::from_icon_name("microphone-sensitivity-muted-symbolic");
        mute_icon.set_pixel_size(14);
        mute_icon.add_css_class("voice-status");
        hbox.append(&mute_icon);

        let deaf_icon = Image::from_icon_name("audio-volume-muted-symbolic");
        deaf_icon.set_pixel_size(14);
        deaf_icon.add_css_class("voice-status");
        hbox.append(&deaf_icon);

        if let Some(volume) = participant.volume {
            let volume_label = Label::new(Some(&format!("{}%", volume)));
            volume_label.add_css_class("volume-indicator");
            hbox.append(&volume_label);
        }

        row.set_child(Some(&hbox));

        let participant_row = Self {
            row,
            name: name_label,
            avatar,
            mute_icon,
            deaf_icon,
        };
        participant_row.update(
            participant,
            match config.overlay.name_display {
                NameDisplayMode::Always => true,
                NameDisplayMode::SpeakingOnly => participant.speaking,
                NameDisplayMode::Never => false,
            },
            config.avatar_size_px(),
        );
        participant_row
    }

    fn update(&self, participant: &Participant, name_visible: bool, avatar_size: i32) {
        self.name.set_text(&participant.username);
        self.name.set_visible(name_visible);
        self.mute_icon.set_visible(participant.mute);
        self.deaf_icon.set_visible(participant.deaf);
        self.avatar.set_size(avatar_size);
        if participant.speaking {
            self.row.add_css_class("speaking");
            self.avatar.widget().add_css_class("speaking");
        } else {
            self.row.remove_css_class("speaking");
            self.avatar.widget().remove_css_class("speaking");
        }
    }
}
