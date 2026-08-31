# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.3.0] - 2026-08-31

### Added
- Universal user-level installer and manager for native Arch Linux clients.
- Install, update, repair, status, doctor, and uninstall commands with explicit
  Vesktop or Discord client selection when both are installed.
- Managed pinned Vencord builds, checksum-verified overlay downloads, and
  automatic enablement of the `VesktopVoiceOverlay` plugin.
- Explicit-target Discord Desktop integration through Vencord's official
  injector and conservative ownership checks for Vesktop and Discord state.

### Changed
- Added installer fixture coverage and CI ShellCheck validation.
- Documented the supported installer workflow, ownership model, recovery rules,
  and native-client limitations.

### Limitations
- Flatpak, AppImage, arbitrary custom installations, and foreign/custom Vencord
  or Discord integrations are detected or refused rather than adopted.
- The plugin remains a Vencord source userplugin built from the pinned source;
  the release `.tgz` is not directly installable through Vesktop.
- The AUR template remains unpublished; AUR publication is deferred.

## [1.2.1] - 2026-08-31

### Fixed
- Custom X/Y changes now activate custom positioning immediately.
- Center positioning no longer pins the overlay to the usable top-left corner.

### Changed
- Public branding is now Discord Voice Overlay; internal plugin, bridge,
  protocol, binary, service, and socket identifiers remain unchanged for
  compatibility.

## [1.2.0] - 2026-08-31

### Added
- Authoritative `clear` state on voice-channel leave, including reconnect replay.
- Center position option and compatibility with camelCase/snake_case speaking events.
- Release artifact checksums and tag-to-package version verification.

### Fixed
- Prevent stale rows after leaving a voice channel or disabling/restarting the plugin.
- Keep participant order deterministic across joins, leaves, duplicate IDs, and
  participant limits while reusing existing GTK rows.
- Refresh changed avatar URLs and prevent stale async avatar results from
  overwriting newer state.
- Enforce socket payload limits before allocation, preserve JSONL
  resynchronization, and reject competing clients that could leave stale state.
- Honor Node socket backpressure and bound GTK commands, socket clients, avatar
  downloads, HTTP response sizes, image dimensions, and decode allocations.
- Repair the tag release Cargo working directory and AUR workspace paths.
- Preserve authoritative settings and clear/snapshot state during connected
  socket backpressure.
- Reject insecure `/tmp` socket fallback when `$XDG_RUNTIME_DIR` is absent.
- Prevent delayed disconnect hides from overtaking reconnect state.
- Make first-time AUR publication, release checksums, and source licensing valid.

### Changed
- Align direct gtk-rs dependencies, remove unused dependencies/build metadata,
  and avoid unintended OpenSSL and unused image-codec linkage.
- Document the plugin artifact accurately as a Vencord source bundle rather
  than a directly installable Vesktop package.

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
