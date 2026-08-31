# Architecture - Discord Voice Overlay

## Public Branding and Compatibility

- The public product name is **Discord Voice Overlay**.
- The canonical source repository is
  `https://github.com/Roddygithub/discord-voice-overlay`.
- The persisted plugin name `VesktopVoiceOverlay`, native bridge lookup,
  `VESKTOP_VOICE_OVERLAY/1.0` protocol header, binary, service, socket, config,
  package, and repository identifiers remain unchanged. Renaming those values
  would risk breaking existing settings, clients, or installations.

## Installer Architecture (Candidate v1.3.0)

- The installer is a user-run Bash manager with `install`, `update`, `repair`,
  `status`, `doctor`, and `uninstall` commands plus `--client`, `--dry-run`,
  and `--yes` options.
- Overlay releases are downloaded to the user-owned
  `~/.local/share/discord-voice-overlay/` tree and verified against the tagged
  `SHA256SUMS`; an existing package-owned or local binary is never overwritten.
- The managed Vencord checkout lives under the same user-owned data tree, is
  cloned from the official repository at the pinned revision above, and is
  atomically rebuilt before replacing the prior managed checkout.
- Vesktop integration changes only its `state.json` `vencordDir` field after
  backing up the file. A different existing custom Vencord path requires
  explicit consent and is restorable; arbitrary source trees are never edited.
- Discord integration uses the pinned Vencord `scripts/runInstaller.mjs` wrapper
  with the official CLI's explicit `--install`/`--uninstall -location` arguments
  after checking the native target and requiring explicit consent. Existing
  injected targets are treated as foreign and are not adopted.
- A human-readable installer state file records ownership, selected client,
  managed paths, revision, integration method, and rollback metadata. It never
  stores tokens, credentials, messages, or voice data.
- State and managed paths are validated before mutation or removal; symlinked
  state, service, client, and managed paths are rejected, and release redirects
  are restricted to HTTPS.
- A live foreign Discord injection is a deliberate stop condition: the manager
  does not provide silent adoption, and acceptance must use a clean target or a
  future explicit migration workflow.
- The current installer candidate must invoke the official Vencord CLI with an
  explicit target for non-interactive operation and must write an absolute
  executable path in `ExecStart`; these are acceptance-blocking requirements.
- Vencord persists renderer plugin state as JSON at
  `<XDG_CONFIG_HOME>/Vencord/settings/settings.json`; native Discord and
  Vesktop share this default location. The installer structurally enables only
  `plugins.VesktopVoiceOverlay.enabled`, writes atomically, and restores its
  attributable backup only when the file is unchanged.
- Live acceptance confirmed the enabled settings are consumed by native
  Discord: the renderer started `VesktopVoiceOverlay`, and the managed overlay
  accepted the same-user client connection and initial messages.
- Human voice acceptance confirmed correct overlay visibility and speaking
  state for both the local user and another participant. The internal plugin
  name shown in Vencord's Plugins UI is a deferred naming/UX follow-up and is
  intentionally unchanged during this lifecycle.
- Exact pinned-source audit confirms this is not currently solvable in the
  project plugin alone: Vencord uses `PluginDef.name` as both display text and
  operational identity across generated plugin/native maps, settings,
  enablement, patches, and UI. `PluginAuthor.id` is unrelated, and
  setting-level `displayName` does not apply to the plugin itself.
- The complete live lifecycle passed: repair, update, uninstall, clean
  reinstall, final uninstall, and legacy-service restoration. Runtime changes
  to shared Vencord settings were preserved rather than attributed back to the
  installer; the original settings snapshot remains available in the approved
  rollback bundle.
- The v1.3.0 release metadata is aligned across the installer, overlay, plugin,
  lockfiles, and AUR template. The release workflow remains tag-driven and
  validates package versions, pinned Vencord discovery, release builds, and
  checksums; AUR publication is conditional on its existing secret.
- The user service is installed only at
  `~/.config/systemd/user/vesktop-voice-overlay.service`; package/system units
  and conflicting user units are left untouched.

## Runtime Data Path

```text
Discord renderer stores
  -> Vencord source userplugin (TypeScript)
  -> versioned settings + voice-state JSONL
  -> Node main-process Unix socket client
  -> same-UID Rust Unix socket server
  -> bounded GTK command channel
  -> keyed participant state
  -> GTK4 + gtk4-layer-shell surface
  -> Wayland compositor
```

The plugin uses `VoiceStateStore` and `UserStore` from the existing Discord
renderer. It does not read a token, open a Gateway connection, inspect message
content, or inject into a game process. The renderer invokes the native socket
helper through Vencord's generated `PluginNative` bridge.

## Plugin

`plugin/src/index.ts` owns settings, Flux subscriptions, plugin lifecycle, and
the renderer-to-native calls. `voiceState.ts` maps the current voice channel to
an authoritative snapshot containing only user ID, display name, Discord CDN
avatar URL, mute/deaf state, and speaking state.

Speaking state is maintained from `SPEAKING` and `STOP_SPEAKING` events. Both
`userId` and `user_id`, plus both speaking-flags spellings, are accepted. Voice
channel changes clear the speaking set. Leaving a channel emits an
authoritative `{"type":"clear"}` message.

`native.ts` runs in Vesktop's main process and owns the Node `net.Socket`.
Connection attempts use 500 ms, 1 s, then a 2 s cap. Outbound writes wait for
the server header and honor Node backpressure. The pending queue is capped at
100 lines; settings and voice state are coalesced without eviction both while
disconnected and backpressured, then replayed settings-first.

The plugin is a Vencord source userplugin. The release `.tgz` is only a compact
source bundle; it is not directly installable from Vesktop's settings.

## Protocol And IPC

The default socket is `$XDG_RUNTIME_DIR/vesktop-voice-overlay.sock`; the plugin
and default overlay fail closed when the runtime directory is absent. An
explicit overlay socket path supports manual protocol clients only. The socket
mode is `0700`. The server requires successful Linux `SO_PEERCRED` retrieval
and an exact current-UID match before sending `VESKTOP_VOICE_OVERLAY/1.0\n`.

Protocol v1 accepts:

- `SnapshotV1` with exact `version: 1`;
- validated `type: "settings"` messages;
- `type: "clear"`.

Input is capped at 64 KiB per line before allocation. Oversized and invalid
UTF-8 lines are discarded through the newline so the stream remains
synchronized. Parse logs contain byte counts only. The server accepts one
same-UID client so state cannot outlive its author. A bounded 256-command
channel backpressures the socket reader instead of growing memory.

## Overlay Threading

Socket accept and reads use blocking standard-library Unix I/O on named worker
threads. No GTK object crosses those threads. Validated commands are consumed
by a GLib local future on GTK's main context, where all window and widget work
occurs.

Client lifecycle is counted defensively. When the client disconnects, a
five-second GLib timeout clears and hides the window unless it reconnects.
Queued hides recheck connection state, so a delayed hide cannot overtake a
completed reconnect. An explicit clear removes rows and cached snapshot state
immediately.

## GTK And Layer Shell

The application uses a non-unique `GtkApplication`; duplicate-instance
ownership is determined synchronously by the Unix socket bind. A stale socket
is replaced, while a connectable socket causes startup to fail.

The layer-shell window uses the overlay layer, no keyboard interactivity, zero
exclusive zone, and preset/custom anchors. An empty GDK input region is applied
on every map for pointer pass-through. The structural GTK nodes are transparent;
the visible name pills provide the dark translucent backdrop. The participant
container remains hidden until at least one row is visible.

Rows are keyed by user ID and reused. Each update computes self-first,
case-insensitive name order with a user-ID tie-break, removes duplicate IDs,
applies the configured participant cap after sorting, and reorders existing
`GtkListBoxRow` objects as needed. Speaking, name visibility, mute/deaf badges,
and 28/40 px avatar size update in place.

## Avatar Pipeline

Avatar fetches run on one shared single-worker Tokio runtime and one shared
reqwest client. URLs must use HTTPS with the exact `cdn.discordapp.com` host;
redirects are disabled. Fetch concurrency is capped at eight, response bodies
at 2 MiB, image dimensions at 256x256, and decoder allocation at 8 MiB.

Decoded RGBA images use a 128-entry FIFO cache keyed by URL. GTK texture
creation and widget mutation happen after returning to the main context. Each
row uses a request generation and explicit loading state: URL changes display a
placeholder immediately, same-URL requests are deduplicated, failures can retry,
and late results cannot affect a newer request even after an `A -> B -> A`
sequence.

## Configuration

The overlay optionally reads
`~/.config/vesktop-voice-overlay/config.toml` once at startup and never writes
it. The socket path and `overlay.max_participants` remain local configuration.
Plugin settings override enabled state, position, coordinates, user/name
display modes, and avatar-size mode in memory. Legacy appearance and numeric
avatar-size fields are ignored.

Editing either custom coordinate selects the `custom` position mode before the
settings snapshot is sent, so coordinate fields cannot silently remain paired
with a preset anchor. On Hyprland, the reported surface Y also includes any
exclusive top-bar area before the requested top margin.

The `center` preset leaves all layer-shell edges unanchored so the compositor
centers the natural-size surface; anchoring all four edges would instead place
the natural-size surface at the usable top-left.

## Distribution And Validation

The repository contains an unpublished Arch source-package template plus a
systemd user unit. CI runs Rust
formatting, clippy, tests under Xvfb, plugin lint/tests, a release build, and a
Vencord build at commit `ef29bbeb6119cfb53d1273ed78147bcc97d91261` using
pnpm 11.9.0 with its frozen lockfile. gtk4-layer-shell source builds are pinned
to commit `1c963c51514581c41b9bdae08cdf69171265cdda`.

GTK initialization is process-global, so CSS parsing, avatar measurement, and
container visibility assertions run in one aggregate Rust test rather than on
concurrent test-harness threads. CI executes that suite under Xvfb.

Tag releases verify Cargo/npm versions, rerun Rust/plugin/pinned-Vencord gates,
build from the tag checkout, publish flat-path SHA-256 checksums, and update the
AUR source checksum when credentials exist and the package has already been
registered. The AUR job uses the required `master` branch and includes the
package-source license, but it cannot create an unregistered AUR package; first
publication requires AUR registration plus the maintainer SSH key. No workflow
publishes without a pushed tag.
- Installer fixture mocks delegate non-fixture JavaScript execution to the
  runner's discovered Node executable, avoiding assumptions about setup-node's
  filesystem location.
- The v1.3.0 candidate passed the complete main-branch CI workflow and both
  CodeQL language analyses after this fixture portability correction.
- Tag `v1.3.0` publishes the dynamically linked overlay binary, plugin source
  bundle, and flat-path `SHA256SUMS`; the release workflow skips only the
  optional AUR update when its maintainer secret is absent.

## Dependency Maintenance

- gtk-rs core crates must move in lockstep to avoid duplicate GLib trees;
- GitHub Actions stay SHA-pinned, with related CodeQL pins updated together;
- npm and release-action majors require deliberate migration and release-path
  validation rather than automatic adoption.

## Deferred Or Human-Only Evidence

- Human validation on 2026-08-31 passed a real multi-user Discord voice call,
  speaking transitions, mute/deaf badges, live avatar refresh, overlay and
  Vesktop restart/reconnect, channel leave clearing, and click-through over a
  game on the current v1.2.0 candidate deployment.
- Human validation also passed custom X/Y movement and position persistence after
  overlay restart/reconnect on the Discord Desktop POC with the shared plugin.
- Layer-shell placement and fullscreen behavior were validated in the same
  Hyprland/game session. Sway, niri, wayfire, GNOME, KDE, and multi-monitor
  behavior remain unverified.
- Multi-monitor placement and exclusive-fullscreen behavior remain unverified.
