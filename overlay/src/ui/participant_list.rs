use anyhow::Result;
use gtk4::prelude::*;
use gtk4::{
    pango, Align, Box, Image, Label, ListBox, ListBoxRow, Orientation, Overlay, SelectionMode,
};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
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
    mute_badge: Image,
    deaf_badge: Image,
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

        let name_display = config.overlay.name_display;
        let desired = Self::desired_participants(&config, snapshot);

        let desired_ids: std::collections::HashSet<_> =
            desired.iter().map(|(id, _)| id.clone()).collect();
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

        for (index, (user_id, participant)) in desired.iter().enumerate() {
            if !rows.contains_key(user_id) {
                let participant_row = ParticipantRow::new(&config, participant);
                self.list_box.append(&participant_row.row);
                rows.insert(user_id.clone(), participant_row);
            }

            let existing = &rows[user_id];
            existing.update(
                participant,
                Self::is_name_visible(name_display, participant),
                config.avatar_size_px(),
            );

            // Reuse the keyed row but move it when joins, leaves, or renamed
            // users change the authoritative sort order.
            if existing.row.index() != index as i32 {
                self.list_box.remove(&existing.row);
                self.list_box.insert(&existing.row, index as i32);
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

    fn desired_participants(config: &Config, snapshot: &Snapshot) -> Vec<(String, Participant)> {
        let self_participant = Self::to_participant(&snapshot.self_);
        let mut desired = Vec::with_capacity(
            snapshot
                .participants
                .len()
                .min(config.overlay.max_participants)
                + 1,
        );
        if Self::is_user_visible(config.overlay.user_display, &self_participant) {
            desired.push((snapshot.self_.user_id.clone(), self_participant));
        }

        let mut unique = HashMap::<&str, &Participant>::new();
        for participant in &snapshot.participants {
            if participant.user_id == snapshot.self_.user_id
                || !Self::is_user_visible(config.overlay.user_display, participant)
            {
                continue;
            }
            unique
                .entry(&participant.user_id)
                .and_modify(|current| {
                    if Self::compare_duplicate(participant, current).is_lt() {
                        *current = participant;
                    }
                })
                .or_insert(participant);
        }
        let mut others: Vec<_> = unique
            .into_iter()
            .map(|(id, participant)| (id.to_string(), participant.clone()))
            .collect();
        others.sort_by(|(left_id, left), (right_id, right)| {
            left.username
                .to_lowercase()
                .cmp(&right.username.to_lowercase())
                .then_with(|| left_id.cmp(right_id))
        });
        others.truncate(config.overlay.max_participants);
        desired.extend(others);
        desired
    }

    fn compare_duplicate(left: &Participant, right: &Participant) -> Ordering {
        left.username
            .to_lowercase()
            .cmp(&right.username.to_lowercase())
            .then_with(|| left.username.cmp(&right.username))
            .then_with(|| left.avatar_url.cmp(&right.avatar_url))
            .then_with(|| left.mute.cmp(&right.mute))
            .then_with(|| left.deaf.cmp(&right.deaf))
            .then_with(|| left.speaking.cmp(&right.speaking))
            .then_with(|| left.volume.cmp(&right.volume))
    }

    fn to_participant(self_: &crate::protocol::ParticipantSelf) -> Participant {
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

        let hbox = Box::new(Orientation::Horizontal, 6);
        hbox.set_margin_top(2);
        hbox.set_margin_bottom(2);
        hbox.set_margin_start(2);
        hbox.set_margin_end(2);

        let avatar_size = config.avatar_size_px();
        let avatar = AvatarWidget::new(&participant.avatar_url, avatar_size);

        // Avatar with mute/deaf badges overlaid on the bottom-right corner.
        let avatar_overlay = Overlay::new();
        avatar_overlay.set_child(Some(avatar.widget()));

        let mute_badge = Image::from_icon_name("microphone-sensitivity-muted-symbolic");
        mute_badge.set_pixel_size(12);
        mute_badge.set_halign(Align::End);
        mute_badge.set_valign(Align::End);
        mute_badge.set_margin_end(1);
        mute_badge.set_margin_bottom(1);
        mute_badge.add_css_class("avatar-badge");
        mute_badge.set_visible(false);
        avatar_overlay.add_overlay(&mute_badge);

        let deaf_badge = Image::from_icon_name("audio-headphones-symbolic");
        deaf_badge.set_pixel_size(12);
        deaf_badge.set_halign(Align::End);
        deaf_badge.set_valign(Align::End);
        deaf_badge.set_margin_end(1);
        deaf_badge.set_margin_bottom(1);
        deaf_badge.add_css_class("avatar-badge");
        deaf_badge.set_visible(false);
        avatar_overlay.add_overlay(&deaf_badge);

        hbox.append(&avatar_overlay);

        let name_label = Label::new(Some(&participant.username));
        name_label.add_css_class("participant-name");
        name_label.set_halign(Align::Start);
        name_label.set_ellipsize(pango::EllipsizeMode::End);
        name_label.set_hexpand(true);
        hbox.append(&name_label);

        row.set_child(Some(&hbox));

        let participant_row = Self {
            row,
            name: name_label,
            avatar,
            mute_badge,
            deaf_badge,
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
        self.avatar.set_size(avatar_size);
        self.avatar.update_url(&participant.avatar_url);

        // Deaf implies mute: show the deaf badge alone when both are active.
        let show_mute = participant.mute && !participant.deaf;
        let show_deaf = participant.deaf;
        self.mute_badge.set_visible(show_mute);
        self.deaf_badge.set_visible(show_deaf);

        if participant.speaking {
            self.row.add_css_class("speaking");
            self.avatar.widget().add_css_class("speaking");
        } else {
            self.row.remove_css_class("speaking");
            self.avatar.widget().remove_css_class("speaking");
        }
    }
}

#[cfg(test)]
pub(super) fn assert_gtk_row_updates() {
    use crate::protocol::{ParticipantSelf, PROTOCOL_VERSION};

    let participant = |id: &str, name: &str, speaking: bool, mute: bool, deaf: bool| Participant {
        user_id: id.into(),
        username: name.into(),
        avatar_url: String::new(),
        mute,
        deaf,
        speaking,
        volume: None,
    };
    let snapshot = |self_speaking: bool, participants: Vec<Participant>| Snapshot {
        version: PROTOCOL_VERSION,
        timestamp: 0,
        self_: ParticipantSelf {
            user_id: "me".into(),
            username: "Me".into(),
            avatar_url: String::new(),
            mute: false,
            deaf: false,
            speaking: self_speaking,
        },
        participants,
    };

    let mut config = Config::default();
    config.overlay.user_display = UserDisplayMode::Always;
    config.overlay.name_display = NameDisplayMode::Always;
    config.overlay.max_participants = 2;
    let config = Rc::new(RefCell::new(config));
    let list = ParticipantList::new(config.clone()).expect("participant list builds");

    assert!(list.update(&snapshot(
        true,
        vec![
            participant("b", "Bob", false, true, false),
            participant("a", "Alice", false, false, true),
        ],
    )));
    let rows = list.rows.borrow();
    let b_row = rows["b"].row.clone();
    assert_eq!(rows["me"].row.index(), 0);
    assert_eq!(rows["a"].row.index(), 1);
    assert_eq!(rows["b"].row.index(), 2);
    assert!(rows["a"].deaf_badge.is_visible());
    assert!(!rows["a"].mute_badge.is_visible());
    assert!(rows["b"].mute_badge.is_visible());
    drop(rows);

    assert!(list.update(&snapshot(
        true,
        vec![
            participant("c", "Cara", false, false, false),
            participant("b", "Aaron", true, false, false),
        ],
    )));
    let rows = list.rows.borrow();
    assert_eq!(rows["b"].row, b_row, "keyed row must be reused");
    assert_eq!(rows["b"].row.index(), 1, "renamed row must move");
    assert_eq!(rows["b"].name.text(), "Aaron");
    assert!(rows["b"].row.has_css_class("speaking"));
    assert!(!rows.contains_key("a"), "departed row must be removed");
    drop(rows);

    config.borrow_mut().overlay.user_display = UserDisplayMode::SpeakingOnly;
    assert!(!list.update(&snapshot(false, Vec::new())));
    assert!(list.rows.borrow().is_empty());
    assert!(list.list_box.first_child().is_none());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{ParticipantSelf, PROTOCOL_VERSION};

    fn make_participant(
        id: &str,
        name: &str,
        speaking: bool,
        mute: bool,
        deaf: bool,
    ) -> Participant {
        Participant {
            user_id: id.to_string(),
            username: name.to_string(),
            avatar_url: String::new(),
            mute,
            deaf,
            speaking,
            volume: None,
        }
    }

    fn snapshot(participants: Vec<Participant>) -> Snapshot {
        Snapshot {
            version: PROTOCOL_VERSION,
            timestamp: 0,
            self_: ParticipantSelf {
                user_id: "me".into(),
                username: "Me".into(),
                avatar_url: String::new(),
                mute: false,
                deaf: false,
                speaking: true,
            },
            participants,
        }
    }

    #[test]
    fn ordering_is_deterministic_for_equivalent_snapshots() {
        let participants = vec![
            make_participant("c", "Charlie", false, false, false),
            make_participant("a", "alice", false, false, false),
            make_participant("b", "Bob", false, false, false),
        ];
        let mut config = Config::default();
        config.overlay.user_display = UserDisplayMode::Always;

        let desired = ParticipantList::desired_participants(&config, &snapshot(participants));
        let ids: Vec<_> = desired.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids, ["me", "a", "b", "c"]);
    }

    #[test]
    fn speaking_state_change_does_not_alter_ordering() {
        let not_speaking = vec![
            make_participant("b", "Bob", false, false, false),
            make_participant("a", "Alice", false, false, false),
        ];
        let speaking = vec![
            make_participant("b", "Bob", true, false, false),
            make_participant("a", "Alice", true, false, false),
        ];

        let mut config = Config::default();
        config.overlay.user_display = UserDisplayMode::Always;
        let s1 = ParticipantList::desired_participants(&config, &snapshot(not_speaking));
        let s2 = ParticipantList::desired_participants(&config, &snapshot(speaking));

        let ids1: Vec<_> = s1.iter().map(|(id, _)| id.as_str()).collect();
        let ids2: Vec<_> = s2.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids1, ids2, "speaking state must not change ordering");
    }

    #[test]
    fn self_sorts_first_regardless_of_username() {
        let participants = vec![
            make_participant("me", "Duplicate self", true, false, false),
            make_participant("a", "Alice", false, false, false),
            make_participant("m", "Bob", false, false, false),
        ];
        let mut config = Config::default();
        config.overlay.user_display = UserDisplayMode::Always;
        let desired = ParticipantList::desired_participants(&config, &snapshot(participants));

        assert_eq!(desired[0].0, "me", "self must always be first");
        assert_eq!(desired[0].1.username, "Me", "duplicate self is ignored");
        assert_eq!(desired[1].1.username, "Alice");
        assert_eq!(desired[2].1.username, "Bob");
    }

    #[test]
    fn max_participants_is_applied_after_sorting_and_deduplication() {
        let mut config = Config::default();
        config.overlay.user_display = UserDisplayMode::Always;
        config.overlay.max_participants = 2;
        let participants = vec![
            make_participant("z", "Zulu", false, false, false),
            make_participant("a", "Alice", false, false, false),
            make_participant("a", "Duplicate Alice", false, false, false),
            make_participant("b", "Bob", false, false, false),
        ];

        let desired =
            ParticipantList::desired_participants(&config, &snapshot(participants.clone()));
        let ids: Vec<_> = desired.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids, ["me", "a", "b"]);
        assert_eq!(desired[1].1.username, "Alice");

        let reversed = ParticipantList::desired_participants(
            &config,
            &snapshot(participants.into_iter().rev().collect()),
        );
        let reversed_values: Vec<_> = reversed
            .into_iter()
            .map(|(id, participant)| (id, participant.username))
            .collect();
        let desired_values: Vec<_> = desired
            .into_iter()
            .map(|(id, participant)| (id, participant.username))
            .collect();
        assert_eq!(reversed_values, desired_values);
    }
}
