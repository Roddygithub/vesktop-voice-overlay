use serde::{Deserialize, Serialize};

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
    pub speaking: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<u8>,
}

pub type Snapshot = SnapshotV1;

impl Snapshot {
    pub fn deserialize(line: &str) -> Option<Snapshot> {
        if line.len() > MAX_PAYLOAD_SIZE {
            return None;
        }
        let snapshot: Snapshot = serde_json::from_str(line).ok()?;
        // Validate protocol version
        if snapshot.version != PROTOCOL_VERSION {
            return None;
        }
        Some(snapshot)
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

pub fn get_socket_path() -> String {
    std::env::var("XDG_RUNTIME_DIR")
        .map(|dir| format!("{}/vesktop-voice-overlay.sock", dir))
        .unwrap_or_else(|_| format!("/tmp/vesktop-voice-overlay-{}.sock", std::process::id()))
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
        assert_eq!(snapshot.self_.speaking, true);
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
}
