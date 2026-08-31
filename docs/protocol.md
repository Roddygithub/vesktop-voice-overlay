# Vesktop Voice Overlay - Socket Protocol v1

## Transport

- **Type**: Unix Domain Socket (AF_UNIX, SOCK_STREAM)
- **Path**: `$XDG_RUNTIME_DIR/vesktop-voice-overlay.sock`; the plugin fails
  closed when the runtime directory is unavailable
- **Permissions**: 0700 (user-only), validated via SO_PEERCRED (UID match)
- **Protocol**: JSON Lines (one client message per line) with version header

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

## Settings Message

Alongside snapshots, the plugin sends its current overlay settings whenever
they change (and once right after connecting):

```json
{
  "type": "settings",
  "settings": {
    "enabled": true,
    "position": "custom",
    "custom_x": 2000,
    "custom_y": 400,
    "user_display": "speaking_only",
    "name_display": "speaking_only",
    "avatar_size_mode": "small"
  }
}
```

Validation (invalid settings messages are dropped):

- `position`: one of `top-left`, `top-right`, `bottom-left`, `bottom-right`,
  `center`, `custom`
- `custom_x` / `custom_y`: integers in `[-32768, 32768]`
- `user_display`: `speaking_only` | `always`
- `name_display`: `speaking_only` | `always` | `never`
- `avatar_size_mode`: `small` | `large` (rendered as 28 px / 40 px)

## Clear Message

Leaving a voice channel is an authoritative state transition. The plugin sends
the following message so the overlay removes all rows immediately instead of
retaining the last snapshot:

```json
{"type":"clear"}
```

The clear replaces the cached snapshot and is replayed after reconnects until
the plugin produces a new channel snapshot.

## Versioning Strategy

- Header: `VESKTOP_VOICE_OVERLAY/1.0\n`
- Overlay accepts **exactly** `version: 1`; any other version is rejected
  (no forward compatibility is implemented)
- Breaking change = major bump -> new tag v2.0.0 = plugin + overlay
- Minor bumps = additive fields (optional, with defaults)

## Reconnection Logic

### Plugin (Client)
- Reconnect backoff: 500 ms doubling, capped at **2 s** (500 ms, 1 s, 2 s,
  2 s, …), unlimited retries while the plugin is enabled
- The latest settings and voice-state lines are coalesced while disconnected
  and under connected-socket backpressure; the total pending queue is capped at
  100 lines without evicting authoritative state
- The latest settings line and the latest snapshot-or-clear line are cached and
  **replayed automatically right after the handshake**, so the overlay is
  repopulated without waiting for new voice activity

### Overlay (Server)
- Single listener; accepts **one connection** and rejects additional clients so
  displayed state cannot outlive the client that authored it
- On client disconnect: overlay clears and hides after a short grace delay
  unless the client reconnects
- Validate SO_PEERCRED UID matches current user

## Security

- Socket file mode 0700, owned by user
- SO_PEERCRED validation: reject connections from different UID
- No network exposure (AF_UNIX only)
- Payload size limit: 64KB per line; oversized lines are consumed without
  retaining attacker-controlled bytes and parsing resumes at the next line
- JSON schema validation on deserialize (reject malformed)
- Parse errors log byte counts only, never payload prefixes

## Example Session

# Overlay starts, creates socket
# Plugin connects
SERVER: VESKTOP_VOICE_OVERLAY/1.0
CLIENT: {"version":1,"timestamp":1692000001000,"self":{"userId":"1","username":"Me","avatarUrl":"","mute":false,"deaf":false,"speaking":false},"participants":[]}
CLIENT: {"version":1,"timestamp":1692000002000,"self":{"userId":"1","username":"Me","avatarUrl":"","mute":false,"deaf":false,"speaking":true},"participants":[]}
CLIENT: {"version":1,"timestamp":1692000003000,"self":{"userId":"1","username":"Me","avatarUrl":"","mute":false,"deaf":false,"speaking":true},"participants":[{"userId":"2","username":"Friend","avatarUrl":"https://...","speaking":false,"volume":75}]}
...
