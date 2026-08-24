# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2026-08-24

### Fixed
- **Avatar resizing (Small/Large) now works with downloaded avatars.**
  `GtkPicture` sizes itself from the paintable's intrinsic size, so the
  configured 28/40 px sizes were silently ignored for real avatars
  (placeholder-only rendering was unaffected). Replaced with `GtkImage` +
  `pixel_size` and an explicit overflow clip for the circular mask.
  Regression-tested at the widget level; validated on Hyprland with
  Guild Wars 2.
- Parse-failure logs no longer dump the raw untrusted payload (which can
  contain usernames/user IDs); they log payload length plus a short
  sanitized prefix instead.

### Changed
- Version alignment across overlay, plugin and packaging to 1.1.0.
- Removed dead code: unused `SpeakingIndicator` widget, unused plugin
  `snapshot.ts` builder, dead `Config::save()`/`get_socket_path()` helpers.
- Documentation corrected to match actual behavior: no config file
  auto-creation, no TOML hot-reload, static speaking highlight (no pulse
  animation), actual reconnect backoff (500 ms doubling, 2 s cap), actual
  protocol version handling, documented settings message and replay
  behavior, actual Vencord userplugin workflow.

### Validated
- End-to-end on Hyprland 0.56 / Wayland with Guild Wars 2
  (windowed/borderless): overlay visibility above the game, pointer
  click-through, game keyboard/mouse focus, speaking show/hide, avatar and
  name rendering, live Small/Large resizing, live position changes,
  service restart recovery (measured ≈0.35 s), second-instance refusal.

## [1.0.0] - 2026-08-16

### Added

- Monorepo structure with `plugin/` and `overlay/`
- Unix socket protocol v1 (JSON Lines)
- Vencord plugin (TypeScript) with voice state extraction
- Overlay application (Rust + GTK4 + layer-shell)
- Click-through Wayland overlay with configurable positioning
- Participant list with avatars, names, speaking indicators
- Static speaking highlight for active speakers
- Automatic reconnection with bounded exponential backoff
- UID validation via SO_PEERCRED
- TOML configuration file support
- AUR packaging support
- CI workflow (GitHub Actions)
- Release workflow with GitHub Releases and AUR auto-update
