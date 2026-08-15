# Vesktop Voice Overlay

A Wayland-native voice activity overlay for Vesktop, built with TypeScript, Rust, and GTK4.

## Description

This project provides a light, highly responsive, and Wayland-native (layer-shell) overlay that displays the current voice channel participants and highlights active speakers in Vesktop.

Unlike other solutions, it **never** accesses, stores, or exports your Discord account token and does not connect separate clients to the Discord Gateway, fully respecting your privacy and security boundaries.

## Architecture

The project consists of two core components running on your local machine:

1. **Vencord Plugin (`plugin/`)** — A lightweight TypeScript plugin running inside Vesktop. It extracts local voice state updates and sends serialized JSON snapshots over a private Unix socket.
2. **Overlay App (`overlay/`)** — A high-performance, click-through GTK4/layer-shell Wayland client written in Rust. It listens to the socket, loads avatars asynchronously, and renders speaking indicators over games or other windows.

```
┌─────────────────┐             Unix Socket             ┌──────────────────┐
│  Vesktop Hôte   │ ──────────────────────────────────► │  Overlay Client  │
│ (Vencord Plugin)│ ◄────────────────────────────────── │  (Rust + GTK4)   │
└─────────────────┘             user-only               └──────────────────┘
```

## Security & Privacy

- **No Discord Token** — We never read or touch your account token.
- **No Self-Bots** — No separate Gateway connections. We only read what Vesktop already has in memory.
- **Local Only** — Communication uses a local, private Unix domain socket under `$XDG_RUNTIME_DIR` with user-only permissions (`0700` / `SO_PEERCRED` UID validation). No open network ports.

## Monorepo Layout

This repository is structured as a monorepo for coordinated development and releases:

```
vesktop-voice-overlay/
├── plugin/           # Vencord plugin (TypeScript/npm)
│   ├── src/          # Source files (index, socket, protocol, voiceState)
│   └── manifest.json # Vencord manifest
├── overlay/          # Overlay client (Rust/Cargo)
│   ├── src/          # Source files (main, layer_shell, socket_server, ui)
│   └── Cargo.toml
├── packaging/
│   └── aur/          # PKGBUILD for Arch Linux AUR packaging
└── memory-bank/       # Durable engineering design (PRD, Tech Stack, Plan)
```

## Distribution Strategy

While developed together inside this monorepo, the components are distributed separately to align with Linux packaging and Vencord standards:

| Component | Target Channel | User Installation |
|-----------|----------------|-------------------|
| **Overlay App (Rust)** | **AUR (Arch Linux)** | `yay -S vesktop-voice-overlay` |
| **Vencord Plugin (JS)** | **Vencord Store / Release** | Install from Vesktop UI (or import `.tgz` release asset) |

The two components are strictly versioned together using Git tags (e.g., `v1.0.0` releases matching plugin + overlay socket protocol compatibility).

## Development Setup

The project uses [BMad Method](https://github.com/bmad-code-org/BMAD-METHOD) for product discovery and follows the [Vibe Coding guide](https://github.com/EnzeD/vibe-coding) with a local memory bank.

### Install pinned dev setup:

```bash
./scripts/bootstrap-bmad.sh
./scripts/bootstrap-vibe-coding.sh
```

The generated BMAD files and the fetched Vibe Coding guide are local tooling and intentionally ignored by Git. See `AGENTS.md` for durable engineering constraints.

### Build instructions:

#### Vencord Plugin:
```bash
cd plugin
npm install
npm run build
npm pack # produces the distributable .tgz
```

#### Overlay App:
```bash
cd overlay
cargo build --release
```

## Upstream Projects

- [Discover Overlay](https://github.com/trigg/Discover) (GPL-3.0) — Design inspiration.
- [Vesktop](https://github.com/Vencord/Vesktop) (GPL-3.0) — Target Discord client.
- [Vencord](https://github.com/Vendicated/Vencord) (GPL-3.0) — Target plugin injector.

No upstream source has been copied into this repository. Any future reuse must preserve copyright notices, attribution, and GPL-3.0 obligations.

## Disclaimer

This project is not affiliated with Discord, Vesktop, Vencord, or Discover Overlay. Client modifications may violate Discord's Terms of Service; use them at your own risk.

## License

GPL-3.0. See `LICENSE`.