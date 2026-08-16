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
- 🖥️ **Wayland-native**: Uses `layer-shell` protocol for click-through overlay on wlroots compositors (Hyprland, sway, niri, etc.)
- ⚡ **Low latency**: Unix socket bridge (< 100ms speaking indicator update)
- 🎮 **Game compatible**: Click-through overlay works over fullscreen XWayland games
- 🔄 **Auto-reconnect**: Exponential backoff reconnection if Vesktop or overlay restarts

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
# 1. Overlay binary (AUR)
yay -S vesktop-voice-overlay

# 2. Vencord Plugin (via Vesktop UI)
# Vesktop → Settings → Plugins → Install from Store → "Vesktop Voice Overlay"
# OR: Install from file → vesktop-voice-overlay-plugin-1.0.0.tgz

# 3. Launch overlay
vesktop-voice-overlay
```

### Manual Build (Any Linux)

#### Prerequisites
```bash
# Arch
sudo pacman -S rust gtk4 libadwaita meson ninja wayland-protocols valac

# Ubuntu/Debian
sudo apt install rustc cargo libgtk-4-dev libadwaita-1-dev meson ninja-build libwayland-dev wayland-protocols valac
```

#### Build Overlay (Rust)
```bash
git clone https://github.com/Roddygithub/vesktop-voice-overlay.git
cd vesktop-voice-overlay/overlay

# Build gtk4-layer-shell from source (required)
git clone https://github.com/wmww/gtk4-layer-shell.git /tmp/gtk4-layer-shell
cd /tmp/gtk4-layer-shell && meson setup build --prefix=/usr/local && ninja -C build && sudo ninja -C build install && sudo ldconfig

# Build overlay
cd ~/vesktop-voice-overlay/overlay
cargo build --release
# Binary at: target/release/vesktop-voice-overlay
```

#### Build Plugin (TypeScript)
```bash
cd ~/vesktop-voice-overlay/plugin
npm ci
npm run build
npm pack  # Produces vesktop-voice-overlay-plugin-1.0.0.tgz
```

#### Install Plugin in Vesktop
1. Open Vesktop → Settings → Plugins
2. Click **"Install from file"**
3. Select `vesktop-voice-overlay-plugin-1.0.0.tgz`

### Run
```bash
# Start overlay (keep running in background)
./target/release/vesktop-voice-overlay

# Or with debug logging
RUST_LOG=debug ./target/release/vesktop-voice-overlay
```

## Configuration

Created automatically at `~/.config/vesktop-voice-overlay/config.toml`:

```toml
[overlay]
position = "top-right"     # top-left, top-right, bottom-left, bottom-right, center, custom
custom_x = 0
custom_y = 0
max_participants = 10
avatar_size = 40

[appearance]
theme = "auto"             # auto, light, dark
speaking_pulse_ms = 1000
show_names = true
```

## Usage

1. Start the overlay (`vesktop-voice-overlay` or `./target/release/vesktop-voice-overlay`)
2. Open Vesktop and join a voice channel
3. Overlay appears automatically with participant avatars
4. **Green pulse ring** = currently speaking
5. **Your avatar** has a small indicator dot

## Auto-start (systemd user service)

```ini
# ~/.config/systemd/user/vesktop-voice-overlay.service
[Unit]
Description=Vesktop Voice Overlay
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/bin/vesktop-voice-overlay  # or /path/to/binary
Restart=on-failure
Environment=XDG_RUNTIME_DIR=/run/user/1000

[Install]
WantedBy=default.target
```

```bash
systemctl --user daemon-reload
systemctl --user enable --now vesktop-voice-overlay
```

## Distribution

| Component | Channel | Install Command |
|-----------|---------|-----------------|
| **Overlay (Rust)** | AUR (Arch Linux) | `yay -S vesktop-voice-overlay` |
| **Plugin (TypeScript)** | Vencord Store / GitHub Releases | Vesktop UI → Install from file |

Both components versioned together via Git tags (`v1.0.0`, `v1.1.0`, etc.) — matching plugin + overlay share compatible socket protocol.

## Development

### Project Structure
```
vesktop-voice-overlay/
├── plugin/                    # Vencord Plugin (TypeScript)
│   ├── src/
│   │   ├── index.ts          # Plugin entry point
│   │   ├── protocol.ts       # Socket protocol types + serialization
│   │   ├── socket.ts         # Unix socket client + reconnection
│   │   ├── snapshot.ts       # Voice state → snapshot builder
│   │   ├── socket.ts         # Unix socket client
│   │   └── vencord/api.ts    # Vencord API types
│   ├── manifest.json         # Vencord manifest
│   ├── package.json
│   └── tsconfig.json
├── overlay/                   # Overlay App (Rust)
│   ├── src/
│   │   ├── main.rs           # GTK4 app entry
│   │   ├── layer_shell.rs    # Wayland layer-shell setup
│   │   ├── socket_server.rs  # Unix socket server + SO_PEERCRED
│   │   ├── lifecycle.rs      # Overlay show/hide logic
│   │   ├── protocol.rs       # Protocol deserialization
│   │   ├── config.rs         # TOML config + hot-reload
│   │   └── ui/               # GTK4 widgets
│   ├── Cargo.toml
│   └── build.rs              # Version embedding from git tags
├── packaging/aur/             # AUR PKGBUILD
├── memory-bank/               # Engineering docs (PRD, Tech Stack, Plan)
├── docs/protocol.md           # Socket protocol v1 spec
├── .github/workflows/         # CI/CD pipelines
└── scripts/                   # Bootstrap scripts
```

### CI/CD Pipeline
- **CI** (`.github/workflows/ci.yml`): Format check, build, test for both components
- **Release** (`.github/workflows/release.yml`): Tag push → builds artifacts → GitHub Release → AUR update

```bash
# Local validation
cd overlay && cargo fmt --check && cargo clippy -- -D warnings && cargo test
cd ../plugin && npm run typecheck && npm run lint && npm test
```

### Socket Protocol v1
See [`docs/protocol.md`](docs/protocol.md) for full spec.

**Handshake:**
```
Server sends: "VESKTOP_VOICE_OVERLAY/1.0\n"
Client validates, then sends JSON Lines snapshots
```

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

Tested on wlroots-based Wayland compositors:
- ✅ **Hyprland** (primary target)
- ✅ **sway**
- ✅ **niri**
- ✅ **wayfire**
- ⚠️ GNOME/KDE (layer-shell support varies)

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
