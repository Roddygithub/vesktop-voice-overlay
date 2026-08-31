# Vesktop Voice Overlay

[![CI](https://github.com/Roddygithub/vesktop-voice-overlay/workflows/CI/badge.svg)](https://github.com/Roddygithub/vesktop-voice-overlay/actions/workflows/ci.yml)
[![Release](https://github.com/Roddygithub/vesktop-voice-overlay/workflows/Release/badge.svg)](https://github.com/Roddygithub/vesktop-voice-overlay/actions/workflows/release.yml)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](https://opensource.org/licenses/GPL-3.0)
[![Version](https://img.shields.io/github/v/tag/Roddygithub/vesktop-voice-overlay?label=version&sort=semver)](https://github.com/Roddygithub/vesktop-voice-overlay/releases)

A **Wayland-native** voice activity overlay for **Vesktop**, built with **TypeScript**, **Rust**, and **GTK4**.

## Overview

A lightweight, highly responsive, **Wayland-native (layer-shell)** overlay that displays current voice channel participants and highlights active speakers in Vesktop.

### Key Features

- 🔒 **Privacy-first**: No Discord token access, no self-bots, no separate Gateway connections
- 🖥️ **Wayland-native**: Uses `layer-shell` protocol with an empty input region, so mouse clicks pass through to whatever is underneath (wlroots compositors: Hyprland, sway, niri, etc.)
- ⚡ **Low latency**: Event-driven voice updates over a local Unix socket
- 🎮 **Game compatible**: Click-through overlay works over fullscreen XWayland games
- 🔄 **Auto-reconnect**: Fast bounded backoff (≤ 2s) if Vesktop or the overlay restarts; the latest settings and voice snapshot are replayed automatically so no voice activity is needed to repopulate the overlay
- 🚀 **Session autostart**: ships a `systemd --user` service (`vesktop-voice-overlay.service`)

## Architecture

```
┌─────────────────┐     Unix Socket ($XDG_RUNTIME_DIR)      ┌──────────────────┐
│  Vesktop Host   │ ──────────────────────────────────────► │  Overlay Client  │
│ (Vencord Plugin)│ ◄────────────────────────────────────── │  (Rust + GTK4)   │
└─────────────────┘            user-only (0700)             └──────────────────┘
```

| Component | Technology | Role |
|-----------|------------|------|
| **Vencord Plugin** | TypeScript / Node.js | Runs inside Vesktop, extracts voice state, sends JSON snapshots via Unix socket |
| **Overlay App** | Rust / GTK4 / layer-shell | Wayland click-through overlay, receives snapshots, renders avatars + speaking indicators |

### Security & Privacy

- ✅ **No Discord token** — Never reads, stores, or transmits your account token
- ✅ **No self-bots** — No separate Discord Gateway connections
- ✅ **Local only** — Unix domain socket under `$XDG_RUNTIME_DIR` with `0700` permissions + `SO_PEERCRED` UID validation
- ✅ **No network ports** — Pure local IPC

## Installation

### Quick Start (Arch Linux / Hyprland)

```bash
# 1. Install build/runtime dependencies
sudo pacman -S rust gtk4 gtk4-layer-shell pkg-config

# 2. Build the overlay
git clone https://github.com/Roddygithub/vesktop-voice-overlay.git
cd vesktop-voice-overlay/overlay
cargo build --release --locked

# 3. Build Vencord with the source userplugin (instructions below), then run
./target/release/vesktop-voice-overlay
```

The repository contains an AUR `PKGBUILD`, but no package is currently
published in the AUR.

### Manual Build (Any Linux)

#### Prerequisites
```bash
# Arch
sudo pacman -S rust gtk4 libadwaita gtk4-layer-shell pkg-config

# Other distros: install rustc/cargo, GTK4, libadwaita and gtk4-layer-shell
# (on distros that do not package gtk4-layer-shell, build it from source:
#  https://github.com/wmww/gtk4-layer-shell)
```

#### Build Overlay (Rust)
```bash
git clone https://github.com/Roddygithub/vesktop-voice-overlay.git
cd vesktop-voice-overlay/overlay
cargo build --release --locked
# Binary at: target/release/vesktop-voice-overlay
```

#### Pack Plugin Source (optional)
```bash
cd ~/vesktop-voice-overlay/plugin
npm ci
npm pack  # Produces a source bundle, not a directly installable Vesktop plugin
```

Vencord does not load arbitrary npm archives from Vesktop's plugin settings.
The plugin is currently distributed as a Vencord source userplugin and must be
included in a custom Vencord build. It has not been accepted into Vencord's
built-in plugin set.

### Development: Vencord userplugin workflow (supported)

This is the workflow used for development and for custom Vesktop builds
(this is how the plugin is actually built and loaded when using a local
Vencord):

```bash
git clone https://github.com/Vendicated/Vencord.git
cd Vencord
git checkout ef29bbeb            # revision pinned by CI (.github/workflows/ci.yml)

mkdir -p src/userplugins/vesktopVoiceOverlay
cp <repo>/plugin/src/{index.ts,native.ts,protocol.ts,resendCache.ts,voiceState.ts} \
   src/userplugins/vesktopVoiceOverlay/

pnpm install --no-frozen-lockfile
pnpm build

# REQUIRED: without this sentinel file, Vesktop considers the dist dir
# invalid and silently downloads stock Vencord over your build at launch.
printf '{}\n' > dist/package.json
```

Then point Vesktop at the build: Developer Settings → Vencord Location →
select `.../Vencord/dist`, and fully restart Vesktop.

Verification: `grep -c VesktopVoiceOverlay dist/vencordDesktopRenderer.js`
and `dist/vencordDesktopMain.js` must both be ≥ 1.

### Run
```bash
# Start overlay (keep running in background)
./target/release/vesktop-voice-overlay

# Or with debug logging
RUST_LOG=debug ./target/release/vesktop-voice-overlay
```

## Configuration

Runtime behavior is driven by the plugin settings in Vesktop
(Vencord plugin options: position, custom X/Y, user display, name display,
avatar size). Changes apply immediately and are replayed automatically after
any restart.

An optional TOML file at `~/.config/vesktop-voice-overlay/config.toml` is
read at overlay startup if present — it is **never created or written** by
the overlay. The `[socket]` path and `[overlay].max_participants` are durable
local options. Other overlay display values act as startup defaults and are
overridden when plugin settings arrive. Legacy `[appearance]` and
`overlay.avatar_size` keys are ignored:

```toml
[socket]
path = "/run/user/1000/vesktop-voice-overlay.sock"   # default: $XDG_RUNTIME_DIR/vesktop-voice-overlay.sock
```

The plugin fails closed if `$XDG_RUNTIME_DIR` is unavailable. An explicit
overlay socket path is only useful for manual protocol clients because the
plugin always uses the runtime-directory path.

## Usage

1. Start the overlay (`systemctl --user start vesktop-voice-overlay` or `./target/release/vesktop-voice-overlay`)
2. Open Vesktop and join a voice channel
3. Overlay appears automatically with participant avatars
4. **Green ring** = currently speaking (static highlight)

## Auto-start (systemd user service)

The packaging template installs `vesktop-voice-overlay.service` in
`/usr/lib/systemd/user/`. Once installed, it starts the overlay with your graphical session,
restarts it if it ever exits, and is independent of Vesktop's lifecycle
(the plugin reconnects whenever Vesktop appears).

```bash
# Enable autostart for every session:
systemctl --user enable --now vesktop-voice-overlay.service

# Manual control:
systemctl --user status vesktop-voice-overlay.service
journalctl --user -u vesktop-voice-overlay.service -f
```

If your compositor session does not activate `graphical-session.target`
(e.g. Hyprland started without uwsm), either start it from your Hyprland
config (`exec-once = systemctl --user start vesktop-voice-overlay`) or enable
the default.target variant:

```bash
systemctl --user enable vesktop-voice-overlay.service
```

Running a second instance manually while the service owns the socket fails
cleanly with `another vesktop-voice-overlay instance owns ...` and exit code 1.

## Distribution

| Component | Channel | Install Command |
|-----------|---------|-----------------|
| **Overlay (Rust)** | Source build; unpublished AUR template | Build with Cargo |
| **Plugin (TypeScript)** | Source userplugin / GitHub source bundle | Build inside pinned Vencord source |

Both components versioned together via Git tags (`v1.0.0`, `v1.1.0`, etc.) — matching plugin + overlay share compatible socket protocol.

## Development

### Project Structure
```
vesktop-voice-overlay/
├── plugin/                    # Vencord Plugin (TypeScript)
│   ├── src/
│   │   ├── index.ts          # Plugin entry point
│   │   ├── protocol.ts       # Socket protocol types + serialization
│   │   ├── native.ts         # Node.js socket client (main process, net)
│   │   ├── resendCache.ts    # Reconnect backoff + settings/snapshot replay
│   │   └── voiceState.ts     # Vencord voice state accessors
│   ├── package.json
│   └── tsconfig.json
├── overlay/                   # Overlay App (Rust)
│   ├── src/
│   │   ├── main.rs           # GTK4 app entry
│   │   ├── layer_shell.rs    # Wayland layer-shell setup
│   │   ├── socket_server.rs  # Unix socket server + SO_PEERCRED
│   │   ├── lifecycle.rs      # Overlay show/hide logic
│   │   ├── protocol.rs       # Protocol deserialization
│   │   ├── config.rs         # Optional TOML config (loaded once at startup)
│   │   └── ui/               # GTK4 widgets
│   └── Cargo.toml
├── packaging/aur/             # AUR PKGBUILD
├── memory-bank/               # Engineering docs (PRD, Tech Stack, Plan)
├── docs/protocol.md           # Socket protocol v1 spec
└── .github/workflows/         # CI/CD pipelines
```

### CI/CD Pipeline
- **CI** (`.github/workflows/ci.yml`): Format check, build, test for both components
- **Release** (`.github/workflows/release.yml`): Tag push → validates and builds
  artifacts → GitHub Release → optional AUR update when credentials are present

```bash
# Local validation
cd overlay && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --locked
cd ../plugin && npm run lint && npm test
```

### Socket Protocol v1
See [`docs/protocol.md`](docs/protocol.md) for full spec.

**Handshake:**
```
Server sends: "VESKTOP_VOICE_OVERLAY/1.0\n"
Client validates, then sends JSON Lines snapshots
```

On voice-channel leave, the plugin sends `{"type":"clear"}` so stale rows are
removed immediately and the clear state is replayed after reconnects.

**Snapshot:**
```json
{
  "version": 1,
  "timestamp": 1692000000000,
  "self": {
    "userId": "123...",
    "username": "You",
    "avatarUrl": "https://cdn.discordapp.com/...",
    "mute": false,
    "deaf": false,
    "speaking": true
  },
  "participants": [
    { "userId": "456...", "username": "Friend", "avatarUrl": "...", "speaking": false, "volume": 80 }
  ]
}
```

## Supported Compositors

Layer-shell support is compositor-dependent:
- ✅ **Hyprland** (primary target) — validated end-to-end on Hyprland 0.56
  with Guild Wars 2 (windowed/borderless): overlay visibility, pointer
  click-through, game focus, speaking show/hide, avatar sizing
- ⚠️ **sway** / **niri** / **wayfire** — expected to work through layer-shell,
  but not individually validated
- ⚠️ GNOME/KDE (layer-shell support varies) — untested
- ⚠️ Multi-monitor placement and exclusive-fullscreen games are untested

## License

GPL-3.0 — See [LICENSE](LICENSE)

Compatible with upstream projects:
- [Discover Overlay](https://github.com/trigg/Discover) (GPL-3.0) — Design inspiration
- [Vesktop](https://github.com/Vencord/Vesktop) (GPL-3.0) — Target client
- [Vencord](https://github.com/Vendicated/Vencord) (GPL-3.0) — Plugin platform

## Disclaimer

> This project is not affiliated with Discord, Vesktop, Vencord, or Discover Overlay. Client modifications may violate Discord's Terms of Service; use at your own risk.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

**Built with** 🦀 Rust + 📘 TypeScript + 🎨 GTK4 + 🌊 Wayland layer-shell
