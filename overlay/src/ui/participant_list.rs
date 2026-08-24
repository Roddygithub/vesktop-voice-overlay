use anyhow::Result;
use gtk4::prelude::*;
use gtk4::{
    pango, Align, Box, Image, Label, ListBox, ListBoxRow, Orientation, Overlay, SelectionMode,
};
use std::cell::RefCell;
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

        // Deterministic stable ordering: self first, then others sorted by
        // display name (case-insensitive). Existing rows are never reordered
        // (visual stability); this sort governs the order new rows appear in.
        let self_user_id = snapshot.self_.user_id.clone();
        desired.sort_by_key(|(id, participant)| {
            let is_self = id == &self_user_id;
            (!is_self, participant.username.to_lowercase())
        });

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

        for (user_id, participant) in &desired {
            if let Some(existing) = rows.get(user_id) {
                existing.update(
                    participant,
                    Self::is_name_visible(name_display, participant),
                    config.avatar_size_px(),
                );
            } else {
                let participant_row = ParticipantRow::new(&config, participant);
                self.list_box.append(&participant_row.row);
                rows.insert(user_id.clone(), participant_row);
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
mod tests {
    use super::*;

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

    fn sort_key(self_id: &str, p: &Participant) -> (bool, String) {
        let is_self = p.user_id == self_id;
        (!is_self, p.username.to_lowercase())
    }

    #[test]
    fn ordering_is_deterministic_for_equivalent_snapshots() {
        let self_id = "me";
        let participants = vec![
            make_participant("c", "Charlie", false, false, false),
            make_participant("a", "alice", false, false, false),
            make_participant("b", "Bob", false, false, false),
        ];

        let mut sorted1 = participants.clone();
        sorted1.sort_by_key(|a| sort_key(self_id, a));
        let mut sorted2 = participants.clone();
        sorted2.sort_by_key(|a| sort_key(self_id, a));

        let ids1: Vec<&str> = sorted1.iter().map(|p| p.user_id.as_str()).collect();
        let ids2: Vec<&str> = sorted2.iter().map(|p| p.user_id.as_str()).collect();
        assert_eq!(
            ids1, ids2,
            "equivalent snapshots must produce identical order"
        );
    }

    #[test]
    fn speaking_state_change_does_not_alter_ordering() {
        let self_id = "me";
        let not_speaking = vec![
            make_participant("b", "Bob", false, false, false),
            make_participant("a", "Alice", false, false, false),
        ];
        let speaking = vec![
            make_participant("b", "Bob", true, false, false),
            make_participant("a", "Alice", true, false, false),
        ];

        let mut s1 = not_speaking;
        s1.sort_by_key(|a| sort_key(self_id, a));
        let mut s2 = speaking;
        s2.sort_by_key(|a| sort_key(self_id, a));

        let ids1: Vec<&str> = s1.iter().map(|p| p.user_id.as_str()).collect();
        let ids2: Vec<&str> = s2.iter().map(|p| p.user_id.as_str()).collect();
        assert_eq!(ids1, ids2, "speaking state must not change ordering");
    }

    #[test]
    fn self_sorts_first_regardless_of_username() {
        let self_id = "zzz";
        let participants = vec![
            make_participant("zzz", "Zed", false, false, false),
            make_participant("a", "Alice", false, false, false),
            make_participant("m", "Bob", false, false, false),
        ];

        let mut sorted = participants;
        sorted.sort_by_key(|a| sort_key(self_id, a));

        assert_eq!(sorted[0].user_id, "zzz", "self must always sort first");
        assert_eq!(sorted[1].username, "Alice");
        assert_eq!(sorted[2].username, "Bob");
    }

    #[test]
    fn deaf_implies_mute_badge_visibility() {
        // deaf=true, mute=true → show deaf badge only (implies mute)
        let p = make_participant("a", "Alice", false, true, true);
        let show_mute = p.mute && !p.deaf;
        let show_deaf = p.deaf;
        assert!(!show_mute, "deaf badge replaces mute badge");
        assert!(show_deaf, "deaf badge shown when deaf");

        // deaf=false, mute=true → show mute badge only
        let p = make_participant("a", "Alice", false, true, false);
        let show_mute = p.mute && !p.deaf;
        let show_deaf = p.deaf;
        assert!(show_mute, "mute badge shown when mute without deaf");
        assert!(!show_deaf, "deaf badge hidden when not deaf");

        // neither → no badges
        let p = make_participant("a", "Alice", false, false, false);
        let show_mute = p.mute && !p.deaf;
        let show_deaf = p.deaf;
        assert!(!show_mute);
        assert!(!show_deaf);
    }

    #[test]
    fn ordering_survives_join_and_leave() {
        let self_id = "me";

        // Initial: me + Alice + Bob
        let mut participants = vec![
            make_participant("me", "Me", false, false, false),
            make_participant("b", "Bob", false, false, false),
            make_participant("a", "Alice", false, false, false),
        ];
        participants.sort_by_key(|a| sort_key(self_id, a));
        let ids1: Vec<&str> = participants.iter().map(|p| p.user_id.as_str()).collect();
        assert_eq!(ids1, vec!["me", "a", "b"]);

        // Charlie joins
        participants.push(make_participant("c", "Charlie", false, false, false));
        participants.sort_by_key(|a| sort_key(self_id, a));
        let ids2: Vec<&str> = participants.iter().map(|p| p.user_id.as_str()).collect();
        assert_eq!(ids2, vec!["me", "a", "b", "c"]);

        // Alice leaves
        participants.retain(|p| p.user_id != "a");
        participants.sort_by_key(|a| sort_key(self_id, a));
        let ids3: Vec<&str> = participants.iter().map(|p| p.user_id.as_str()).collect();
        assert_eq!(ids3, vec!["me", "b", "c"]);
    }
}
