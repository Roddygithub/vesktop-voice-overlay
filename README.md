# Vesktop Voice Overlay

A Wayland-native voice activity overlay for Vesktop.

The project will display the current voice participants and highlight active
speakers without reading, storing, or exporting a Discord account token. The
planned design combines a small Vencord plugin, a private local Unix socket,
and a GTK/layer-shell overlay inspired by Discover Overlay.

## Status

Planning stage. No working overlay has been released yet.

## Initial Scope

- Show avatar, display name, and speaking state.
- Run as a click-through layer-shell surface on wlroots compositors.
- Exchange versioned snapshots over a user-private Unix socket.
- Recover cleanly when Vesktop or the overlay restarts.
- Validate the overlay over a fullscreen XWayland game on Hyprland.

## Security Boundaries

- Never access or export the Discord account token.
- Never connect a separate client to the Discord Gateway.
- Never expose voice state on a network socket.
- Export only the voice data needed by the overlay.
- Keep the runtime socket under `$XDG_RUNTIME_DIR` with user-only access.

## Development Method

The project uses [BMad Method](https://github.com/bmad-code-org/BMAD-METHOD)
for product discovery, architecture, implementation, and review. Install the
pinned project setup with:

```bash
./scripts/bootstrap-bmad.sh
```

The generated BMAD files are local tooling and are intentionally ignored by
Git. See `AGENTS.md` for the durable engineering constraints.

## Upstream Projects

- [Discover Overlay](https://github.com/trigg/Discover), GPL-3.0
- [Vesktop](https://github.com/Vencord/Vesktop), GPL-3.0
- [Vencord](https://github.com/Vendicated/Vencord), GPL-3.0

No upstream source has been copied into this repository yet. Any future reuse
must preserve copyright notices, attribution, and GPL-3.0 obligations.

## Disclaimer

This project is not affiliated with Discord, Vesktop, Vencord, or Discover
Overlay. Client modifications may violate Discord's Terms of Service; use them
at your own risk.

## License

GPL-3.0. See `LICENSE`.
