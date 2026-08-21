# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-08-16

### Added

- Monorepo structure with `plugin/` and `overlay/`
- Unix socket protocol v1 (JSON Lines)
- Vencord plugin (TypeScript) with voice state extraction
- Overlay application (Rust + GTK4 + layer-shell)
- Click-through Wayland overlay with configurable positioning
- Participant list with avatars, names, speaking indicators
- Pulse animation for active speakers
- Automatic reconnection with exponential backoff
- UID validation via SO_PEERCRED
- TOML configuration with hot-reload
- AUR packaging support
- CI workflow (GitHub Actions)
- Release workflow with GitHub Releases and AUR auto-update
