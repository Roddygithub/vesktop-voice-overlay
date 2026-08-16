# Vesktop Voice Overlay - Socket Protocol v1

## Transport

- **Type**: Unix Domain Socket (AF_UNIX, SOCK_STREAM)
- **Path**: $XDG_RUNTIME_DIR/vesktop-voice-overlay.sock (fallback: /tmp/vesktop-voice-overlay-$UID.sock)
- **Permissions**: 0700 (user-only), validated via SO_PEERCRED (UID match)
- **Protocol**: JSON Lines (one snapshot per line) with version header

## Handshake

Client (Plugin) connects -> Server (Overlay) accepts
Server sends: "VESKTOP_VOICE_OVERLAY/1.0\n"
Client validates version prefix, then sends snapshots as JSON Lines

## Message Format

### Header (sent once by server on accept)
VESKTOP_VOICE_OVERLAY/1.0\n

### Snapshot (JSON Lines, one per voice state change)
{
  "version": 1,
  "timestamp": 1692000000000,
  "self": {
    "userId": "123456789012345678",
    "username": "Roddy",
    "avatarUrl": "https://cdn.discordapp.com/avatars/123/abc.png",
    "mute": false,
    "deaf": false,
    "speaking": true
  },
  "participants": [
    {
      "userId": "987654321098765432",
      "username": "Friend",
      "avatarUrl": "https://cdn.discordapp.com/avatars/987/def.png",
      "speaking": false,
      "volume": 80
    }
  ]
}

## Type Definitions

### Version 1 (Current)

TypeScript (plugin/src/protocol.ts):
export interface SnapshotV1 {
  version: 1;
  timestamp: number;
  self: ParticipantSelf;
  participants: Participant[];
}

export interface ParticipantSelf {
  userId: string;
  username: string;
  avatarUrl: string;
  mute: boolean;
  deaf: boolean;
  speaking: boolean;
}

export interface Participant {
  userId: string;
  username: string;
  avatarUrl: string;
  speaking: boolean;
  volume?: number;
}

Rust (overlay/src/protocol.rs):
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotV1 {
    pub version: u8,
    pub timestamp: u64,
    pub self: ParticipantSelf,
    pub participants: Vec<Participant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantSelf {
    pub user_id: String,
    pub username: String,
    pub avatar_url: String,
    pub mute: bool,
    pub deaf: bool,
    pub speaking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub user_id: String,
    pub username: String,
    pub avatar_url: String,
    pub speaking: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<u8>,
}

## Versioning Strategy

- Header: VESKTOP_VOICE_OVERLAY/<major>.<minor>\n
- Overlay accepts <= current_version (forward compatible)
- Breaking change = major bump -> new tag v2.0.0 = plugin + overlay
- Minor bumps = additive fields (optional, with defaults)

## Reconnection Logic

### Plugin (Client)
- Exponential backoff: 1s, 2s, 4s, 8s, 16s, max 30s
- Max 5 rapid retries, then 30s interval
- On connect: validate header version, then resume sending

### Overlay (Server)
- Single listener, accepts multiple connections (only first active)
- On client disconnect: hide overlay, wait for new connection
- Validate SO_PEERCRED UID matches current user

## Security

- Socket file mode 0700, owned by user
- SO_PEERCRED validation: reject connections from different UID
- No network exposure (AF_UNIX only)
- Payload size limit: 64KB per snapshot (prevent DoS)
- JSON schema validation on deserialize (reject malformed)

## Example Session

# Overlay starts, creates socket
# Plugin connects
SERVER: VESKTOP_VOICE_OVERLAY/1.0
CLIENT: {"version":1,"timestamp":1692000001000,"self":{"userId":"1","username":"Me","avatarUrl":"","mute":false,"deaf":false,"speaking":false},"participants":[]}
CLIENT: {"version":1,"timestamp":1692000002000,"self":{"userId":"1","username":"Me","avatarUrl":"","mute":false,"deaf":false,"speaking":true},"participants":[]}
CLIENT: {"version":1,"timestamp":1692000003000,"self":{"userId":"1","username":"Me","avatarUrl":"","mute":false,"deaf":false,"speaking":true},"participants":[{"userId":"2","username":"Friend","avatarUrl":"https://...","speaking":false,"volume":75}]}
...
