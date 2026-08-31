# Tech Stack - Discord Voice Overlay

## Runtime

| Component | Technology | Candidate baseline |
|---|---|---|
| Vencord userplugin | TypeScript, Vencord internal APIs | Vencord `ef29bbeb6119cfb53d1273ed78147bcc97d91261` |
| Native socket helper | Node.js `net`, Electron IPC | Node 22 in integration CI |
| Overlay | Rust 2021, GTK4, gtk4-layer-shell | Rust 1.97 in CI |
| IPC | Unix stream socket, JSON Lines | Protocol header `VESKTOP_VOICE_OVERLAY/1.0` |
| Service | systemd user unit | graphical-session target |

## Rust Dependencies

The direct GTK stack is aligned on gtk-rs 0.22 (`gtk4`/`gdk4` 0.11 and
`glib`/`gdk-pixbuf` 0.22). `gtk4-layer-shell` 0.8 provides layer-shell
integration.

Other runtime crates:

- `serde`/`serde_json` for explicit protocol types;
- `tokio` for the shared avatar I/O runtime and bounded command channel;
- `reqwest` with rustls only, no default native-TLS/OpenSSL client;
- `image` with PNG/GIF codecs only and explicit decoder limits;
- `tracing` for local operational logs;
- `clap` for CLI arguments and package-version output;
- `toml`/`dirs` for optional startup configuration.

Socket accept/read logic is intentionally blocking on dedicated standard
threads. GTK work remains on GLib's main context.

## Plugin Tooling

The standalone `plugin/` package supplies ESLint 8, typescript-eslint 6,
TypeScript 5, and Vitest 4 for local lint and pure/native socket tests. It is
not the production compiler. Production compatibility is established by
copying the five runtime `.ts` files into Vencord's `src/userplugins` tree and
running the pinned Vencord build with pnpm 11.9.0 and its frozen lockfile.

The npm `.tgz` contains the five runtime source files, package metadata, and the
GPL-3.0 license. It is a release source bundle, not a Vesktop-installable plugin
or a Vencord store artifact.

## Build And Release

Authoritative local gates:

```bash
cd overlay
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked
cargo build --release --locked
cargo build --profile release-debug --locked

cd ../plugin
npm ci
npm test
npm run lint
npm audit
```

The release profile is optimized, LTO-enabled, stripped, and uses
`panic = "abort"`. `release-debug` inherits the same runtime behavior but keeps
debug information. The panic hook prints thread, panic text, source location,
and a backtrace when `RUST_BACKTRACE` or `RUST_LIB_BACKTRACE` is present. The
systemd unit enables `RUST_BACKTRACE=1`.

CI pins action commits, Vencord source, pnpm, and gtk4-layer-shell source.
Ubuntu package repositories and the Rust/npm registry lockfiles remain external
inputs. Tag releases produce a dynamically linked Linux binary, a plugin source
bundle, and `SHA256SUMS`. The AUR package builds the overlay from the tagged
source archive.

## Supported Environment

The product requires Linux, Wayland, GTK4, and a compositor implementing the
layer-shell protocol. Hyprland is the primary and only recorded end-to-end
target. Other layer-shell compositors are plausible but not release-proven.
GNOME/KDE support is not claimed.

## Intentional Deferrals

- ESLint flat config and current typescript-eslint majors;
- release-action major upgrades;
- broader compositor and multi-monitor support;
- a supported end-user plugin distribution channel beyond custom Vencord
  source builds;
- automated real Discord voice-call testing.

## Installer Candidate

The v1.3.0 installer candidate is a Bash user-level manager. It uses the
standard `git`, `pnpm`, `curl`, `sha256sum`, `node`, and `systemctl --user`
commands; it does not put Vencord or Discord mutation into pacman hooks. The
managed Vencord revision remains
`ef29bbeb6119cfb53d1273ed78147bcc97d91261`.
