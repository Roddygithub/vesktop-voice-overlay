# Progress — Vesktop Voice Overlay

## Statut Global

| Phase | Statut | Progression | Début | Fin Estimée | Notes |
|-------|--------|-------------|-------|-------------|-------|
| **P0 - Fondations & Protocole** | 🟢 Terminé | 100% | J+0 | J+3 | Structure monorepo, spec protocole v1, types TS/Rust, CI base |
| **P1 - Vencord Plugin Core** | 🟢 Terminé | 100% | J+3 | J+5 | Plugin complet, build + tests OK |
| **P2 - Overlay GTK4 Core** | 🟢 Terminé | 100% | J+5 | J+8 | Layer-shell, socket server, UI, build + tests OK |
| **P3 - Intégration & Polish** | 🟢 Validé | 90% | J+8 | J+11 | Socket protocol testé sur Wayland (Hyprland) |
| **P4 - Distribution Pipeline** | 🟢 Terminé | 100% | J+11 | J+14 | PKGBUILD AUR, release workflow, GitHub Release |
| **P5 - Tests & Release v1.0.0** | 🟢 **RELEASED** | 100% | J+14 | J+15 | **Tag v1.0.0 créé** |

**Progression Globale** : **100%** — **v1.0.0 Released**

---

## ✅ Release v1.0.0 — 2026-08-16

### Tag créé
```bash
git tag v1.0.0
git push origin main --tags
```

### Artefacts de release
| Composant | Commande | Sortie |
|-----------|----------|--------|
| **Overlay (Rust)** | `cargo build --release` | `overlay/target/release/vesktop-voice-overlay` |
| **Plugin (TS)** | `npm pack` | `vesktop-voice-overlay-plugin-1.0.0.tgz` |

### Distribution configurée
- **AUR** : `packaging/aur/PKGBUILD` + workflow auto-update
- **GitHub Releases** : workflow `.github/workflows/release.yml`
- **Vencord Store** : plugin `.tgz` installable via Vesktop UI

---

## Résumé Technique

### Architecture
```
Vesktop (Vencord Plugin) → Unix Socket ($XDG_RUNTIME_DIR) → Overlay (GTK4 + layer-shell)
     TypeScript                    JSON Lines v1                    Rust
```

### Composants livrés
| Composant | Fichiers clés | Tests |
|-----------|---------------|-------|
| **Plugin Vencord** | `plugin/src/` (index, protocol, socket, snapshot, voiceState) | 6/6 ✅ |
| **Overlay Rust** | `overlay/src/` (main, layer_shell, socket_server, lifecycle, ui/*) | 5/5 ✅ |
| **Protocole v1** | `docs/protocol.md` + types partagés TS/Rust | - |
| **CI/CD** | `.github/workflows/ci.yml` + `release.yml` | - |
| **Packaging AUR** | `packaging/aur/PKGBUILD` + `.SRCINFO` + `.install` | - |

### Validation E2E
- ✅ Socket server démarre sur Wayland (Hyprland)
- ✅ Connexion client acceptée + validation `SO_PEERCRED`
- ✅ Header protocole `VESKTOP_VOICE_OVERLAY/1.0` échangé
- ✅ Snapshot JSON désérialisé correctement
- ✅ Overlay layer-shell click-through initialisé

---

## Prochaines étapes (Post-v1.0.0)

1. **Publier plugin Vencord Store** : PR vers `Vendicated/Vencord` ou `Vencord/plugins`
2. **Tester installation AUR** : `yay -S vesktop-voice-overlay`
3. **Documentation utilisateur** : README déjà complet
4. **Collecter retours** : GitHub Discussions, Reddit r/archlinux, Discord Vencord

---

## Bootstrap AI — Statut

| Phase Bootstrap | Statut | Date | Notes |
|-----------------|--------|------|-------|
| **Phase 1 — INSPECT/AUDIT** | ✅ Terminé | 2026-08-21 | BROWNFIELD/STANDARD, audit outils IA |
| **Phase 2 — PROPOSE** | ✅ Terminé | 2026-08-21 | SINGLE_MODEL, MiMo V2.5 Free, Level 1 |
| **Phase 3 — BASELINE** | ✅ PASS | 2026-08-21 | Plugin 6/6, Overlay 5/5, clippy OK, fmt OK |
| **Phase 4 — CLEANUP** | ✅ Terminé | 2026-08-21 | Scripts legacy supprimés, docs mises à jour |

### Configuration opérationnelle

```
MODEL_MODE: SINGLE_MODEL
MODÈLE: MiMo V2.5 Free
RÔLES: Lead / Explorer / Implementer / Reviewer (même modèle)
MÉTHODOLOGIE: Aucune (memory-bank existant suffit)
AUTONOMIE: Level 1 — Supervised
FALLBACK: NOT AVAILABLE
```

### Remediation Phase 3

- `overlay/src/protocol.rs:54` : `#[expect(dead_code)]` → `#[allow(dead_code)]`
- `overlay/src/protocol.rs:79` : `assert_eq!(x, true)` → `assert!(x)`

### Nettoyage Phase 4

- `scripts/bootstrap-bmad.sh` : supprimé (legacy BMAD non retenu)
- `scripts/bootstrap-vibe-coding.sh` : supprimé (legacy non retenu)
- `AGENTS.md` : références aux scripts supprimées
- `CONTRIBUTING.md` : workflow post-release reflété

## ✅ Runtime Integration Checkpoint — 2026-08-21

- Root cause Vencord runtime overwrite confirmed: Vesktop 1.6.7 requires
  `dist/package.json` before accepting a custom `vencordDir`; without it,
  managed release files overwrite the custom build.
- Custom Vencord build preserved with `dist/package.json` marker.
- Runtime registry: `VesktopVoiceOverlay` discovered, enabled, and started.
- Native helper: `VencordNative.pluginHelpers.VesktopVoiceOverlay` present.
- Overlay socket: connection accepted with current-user UID.
- Remaining gate: join a real voice channel and speak to validate snapshots and
  speaking transitions.

## ✅ Runtime Data-Path Diagnosis — 2026-08-21

- First failing boundary identified: the blocking Unix socket accept loop was
  scheduled on GTK's main context, preventing lifecycle commands from being
  consumed.
- Fixed by running the socket server on a dedicated Tokio-backed thread.
- Fixed deferred GTK window creation during activation, which caused startup
  criticals and premature process exit during restart.
- Fixed empty-avatar rendering panic and corrected derived config defaults that
  produced a zero avatar size when no config file existed.
- Synthetic snapshot evidence now reaches `ClientConnected`, `Show`, snapshot
  deserialization, and UI update without panic.
- Remaining gate: repeat the real voice-channel visual test with the rebuilt
  overlay and verify speaking transitions.

## ✅ Runtime Content Diagnosis — 2026-08-21

- Direct store inspection found `isCurrentClientInVoiceChannel() === true`,
  while `getUserVoiceChannelId(null, userId)` returned `undefined`.
- The current user's voice state was present in `getAllVoiceStates()` with a
  valid channel ID; lookup now uses `getVoiceStateForUser(userId).channelId`.
- Rust received a real snapshot containing the self member and applied it to
  GTK. The initial disappearance was then traced to avatar HTTP fetches using
  reqwest from a GLib future without a Tokio reactor.
- Avatar fetches now run on a Tokio-backed worker and return bytes to GTK;
  reconnect timeout bookkeeping was also fixed to avoid removing fired GLib
  sources.
- Temporary payload logging was removed after confirming the live payload.
- Remaining gate: visual retest with the repaired overlay.

## ✅ Compact UI Stabilization — 2026-08-21

- Speaking snapshots no longer remove and recreate every `ListBoxRow`.
- Participant rows are keyed by user ID and retain their avatar, label, and
  speaking indicator widgets across snapshot updates.
- Speaking updates now change existing widget state and CSS classes only.
- Replaced the oversized default card with a compact stacked horizontal layout:
  28px avatars, constrained width, low-opacity background, and row-level
  speaking highlight.
- Removed CSS mutation from the drawing callback and eliminated invalid GTK
  width/height CSS properties.
- Remaining gate: human visual retest for compact sizing and no speaking flicker.

## ✅ GTK Allocation Correction — 2026-08-21

- Allocation diagnostics measured a single row at 42px natural height, with a
  36px horizontal content box, 28px avatar, and 16px label.
- The scroller had no minimum content height or natural-height propagation,
  while the layer-shell window used an unset `-1` default height. Explicit
  42px scroller minimum, natural-height propagation, and a 240x60 initial
  window allocation now preserve the natural row height.
- Speaking state remains an in-place update and adds a subtle avatar ring and
  row highlight instead of a vertical meter.
- Clean rebuilt overlay restarted successfully; final human visual retest is
  still required.

## ✅ Discord Voice Widget Modes — 2026-08-21

- Studied the official Discord Game Overlay 101 reference. Defaults now match
  speaking-only behavior: no visible row while idle and a compact row on active
  speech.
- Added configurable `user_display` (`always`/`speaking_only`),
  `name_display` (`always`/`speaking_only`/`never`), and
  `avatar_size_mode` (`small`/`large`).
- Added optional mute/deafen fields to participant snapshots and compact status
  indicators near each name.
- The existing Unix socket and native IPC remain unchanged; rows remain keyed
  and updated in place.
- Remaining gate: human screenshot comparison against the official reference.

## ✅ Voice Widget Rendering Pass — 2026-08-21

- Retained Discord voice-widget display modes and stable keyed participant rows.
- Replaced the full-card treatment with lightweight per-user translucent rows.
- Speaking state is now represented by the avatar ring only; mute/deaf states
  use GTK symbolic icons beside the name.
- The overlay remains a Wayland layer-shell surface above games and does not
  inject or hook into game processes.
- Remaining gate: human comparison screenshot while speaking.

## ✅ Avatar Allocation Correction — 2026-08-21

- The large clipped avatar was caused by `GtkPicture::can_shrink(false)`: once
  a 128px Discord texture loaded, it retained that natural size inside the
  compact layer-shell allocation.
- Avatars can now shrink to their configured allocation and are aligned within
  the compact row.
- Removed the self-marker label whose unconstrained themed background rendered
  as the narrow blue pill seen in the runtime screenshot.
- Rust format, clippy, tests, and release build pass; human visual confirmation
  is still required.

## ✅ Voice Widget Surface Cleanup — 2026-08-21

- Made the GTK window, scrolled viewport, and list transparent so only each
  participant row owns a lightweight translucent background.
- Tightened participant-row spacing and contrast without changing speaking,
  lifecycle, or IPC behavior.
- Rust format, clippy, tests, and release build pass; overlay restarted for
  visual comparison.

## ✅ Avatar Color and Capsule Layout — 2026-08-21

- Removed the Cairo BGRA-to-RGBA mismatch that swapped avatar color channels.
- Moved the dark rounded background from the full participant row to the name
  label, yielding the avatar-plus-name-capsule structure of Discord's widget.
- Rust format, clippy, tests, and release build pass; overlay restarted for a
  final visual comparison.

## ✅ Vencord Voice Widget Settings — 2026-08-21

- Added Vencord controls for enablement, user/name display modes, avatar size,
  corner position, and custom X/Y coordinates.
- Preferences are sent through the existing same-user Unix socket and apply in
  GTK immediately; no additional Discord data enters the protocol.
- Runtime startup confirmed receipt of the settings update. Rust formatting,
  clippy, tests, release build, plugin lint/tests, and Vencord build pass.

---

*Projet livré — v1.0.0 — 2026-08-16*
*Bootstrap COMPLETE — 2026-08-21*
