use serde::{Deserialize, Serialize};

use crate::config::OverlaySettings;

pub const PROTOCOL_VERSION: u8 = 1;
pub const PROTOCOL_HEADER: &str = "VESKTOP_VOICE_OVERLAY/1.0\n";
pub const MAX_PAYLOAD_SIZE: usize = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnapshotV1 {
    pub version: u8,
    pub timestamp: u64,
    #[serde(rename = "self")]
    pub self_: ParticipantSelf,
    pub participants: Vec<Participant>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParticipantSelf {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub username: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: String,
    pub mute: bool,
    pub deaf: bool,
    pub speaking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Participant {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub username: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: String,
    #[serde(default)]
    pub mute: bool,
    #[serde(default)]
    pub deaf: bool,
    pub speaking: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<u8>,
}

pub type Snapshot = SnapshotV1;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Settings { settings: OverlaySettings },
    Clear,
}

impl Snapshot {
    pub fn deserialize(line: &str) -> Option<Snapshot> {
        if line.len() > MAX_PAYLOAD_SIZE {
            return None;
        }
        let snapshot: Snapshot = serde_json::from_str(line).ok()?;
        if snapshot.version != PROTOCOL_VERSION {
            return None;
        }
        Some(snapshot)
    }

    #[cfg(test)]
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

pub fn deserialize_client_message(line: &str) -> Option<ClientMessage> {
    if line.len() > MAX_PAYLOAD_SIZE {
        return None;
    }

    let message: ClientMessage = serde_json::from_str(line).ok()?;
    match &message {
        ClientMessage::Settings { settings } if settings.is_valid() => Some(message),
        ClientMessage::Settings { .. } => None,
        ClientMessage::Clear => Some(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_valid_snapshot() {
        let json = r#"{"version":1,"timestamp":1692000000000,"self":{"userId":"123","username":"Test","avatarUrl":"","mute":false,"deaf":false,"speaking":true},"participants":[{"userId":"456","username":"Friend","avatarUrl":"","speaking":false,"volume":80}]}"#;

        let snapshot = Snapshot::deserialize(json).expect("Should deserialize");
        assert_eq!(snapshot.version, 1);
        assert_eq!(snapshot.timestamp, 1692000000000);
        assert_eq!(snapshot.self_.user_id, "123");
        assert!(snapshot.self_.speaking);
        assert_eq!(snapshot.participants.len(), 1);
        assert_eq!(snapshot.participants[0].volume, Some(80));
    }

    #[test]
    fn test_deserialize_invalid_json() {
        let result = Snapshot::deserialize("not json");
        assert!(result.is_none());
    }

    #[test]
    fn test_deserialize_wrong_version() {
        let json = r#"{"version":2,"timestamp":0,"self":{},"participants":[]}"#;
        let result = Snapshot::deserialize(json);
        assert!(result.is_none());
    }

    #[test]
    fn test_serialize_round_trip() {
        let original = Snapshot {
            version: 1,
            timestamp: 1234567890,
            self_: ParticipantSelf {
                user_id: "123".into(),
                username: "Test".into(),
                avatar_url: "".into(),
                mute: false,
                deaf: false,
                speaking: true,
            },
            participants: vec![Participant {
                user_id: "456".into(),
                username: "Friend".into(),
                avatar_url: "".into(),
                mute: false,
                deaf: false,
                speaking: false,
                volume: Some(75),
            }],
        };

        let json = original.serialize();
        let parsed = Snapshot::deserialize(&json).expect("Should parse");
        assert_eq!(parsed, original);
    }

    #[test]
    fn test_payload_size_limit() {
        let large_json = "x".repeat(MAX_PAYLOAD_SIZE + 1);
        let result = Snapshot::deserialize(&large_json);
        assert!(result.is_none());
    }

    #[test]
    fn test_deserialize_valid_settings() {
        let json = r#"{"type":"settings","settings":{"enabled":true,"position":"custom","custom_x":120,"custom_y":80,"user_display":"speaking_only","name_display":"always","avatar_size_mode":"small"}}"#;

        assert!(matches!(
            deserialize_client_message(json),
            Some(ClientMessage::Settings { .. })
        ));
    }

    #[test]
    fn test_deserialize_clear_message() {
        assert!(matches!(
            deserialize_client_message(r#"{"type":"clear"}"#),
            Some(ClientMessage::Clear)
        ));
    }

    #[test]
    fn test_reject_invalid_settings_position() {
        let json = r#"{"type":"settings","settings":{"enabled":true,"position":"somewhere","custom_x":0,"custom_y":0,"user_display":"speaking_only","name_display":"always","avatar_size_mode":"small"}}"#;

        assert!(deserialize_client_message(json).is_none());
    }

    #[test]
    fn test_deserialize_settings_accepts_large_avatar_mode() {
        let json = r#"{"type":"settings","settings":{"enabled":true,"position":"top-right","custom_x":20,"custom_y":20,"user_display":"speaking_only","name_display":"always","avatar_size_mode":"large"}}"#;

        match deserialize_client_message(json) {
            Some(ClientMessage::Settings { settings }) => {
                assert_eq!(
                    settings.avatar_size_mode,
                    crate::config::AvatarSizeMode::Large
                );
            }
            other => panic!("Expected valid settings message, got {other:?}"),
        }
    }
}
