# Progress — Vesktop Voice Overlay

## Statut Global

## ✅ Suppression du cadre GTK autour des participants — 2026-08-26

- **Cause racine** : GTK CSS ne prend pas en charge `!important`. Les resets de
  transparence étaient rejetés avec `Theme parser error`, laissant Adwaita
  peindre le fond de la fenêtre, du viewport et des rows.
- **Fix** : provider CSS à priorité 800, sans `!important`, conteneur et nœuds
  structurels transparents ; seul le pill du nom conserve un fond.
- **Validation** : aucun `Theme parser error` après redémarrage, 29/29 tests
  Rust passent, binaire release déployé dans `~/.local/bin/`.

## ✅ Fix panneau vide au démarrage — 2026-08-25 (post-release v1.1.0)

- **Symptôme** : rectangle noir opaque ~236×58 px ancré en haut à droite dès le
  login (avant tout lancement de Vesktop), permanent.
- **Cause racine** : `window.present()` au démarrage (main.rs) garde le process
  vivant, mais le `ScrolledWindow` impose `min_content_width(220)` /
  `min_content_height(42)` (ui/mod.rs) → le `.overlay-container`
  (`alpha(#111214, 0.88)`, style.css) garde une boîte naturelle 236×58 même
  totalement vide, et le panneau sombre se rendait tel quel sur le bureau.
- **Fix** : `container.set_visible(false)` à la construction de `OverlayUI` ;
  `update_from_snapshot` / `update_settings` basculent la visibilité du
  conteneur sur le booléen retourné (conteneur visible ⇔ participants visibles).
- **Test régression** : `ui::tests::container_visibility_tracks_visible_participants`
  (GTK-gated, skip gracieux sans display ; fenêtre non associée à
  GApplication pour éviter le Gtk-CRITICAL pre-startup, `window.present()` car
  `is_visible()` tient compte des ancêtres). 29/29 tests OK, fmt + clippy OK.
- **Déploiement local** : binaire release recopié vers `~/.local/bin/`
  (stop service → cp → start, sinon « Text file busy »), service redémarré.
  `hyprctl layers` : surface gtk4-layer-shell désormais `a: 0` (transparente),
  capture grim confirme plus rien de visible, click-through conservé.

## ✅ Release v1.1.0 — 2026-08-24 (Game-ready)

- **Tag** : `v1.1.0` → commit `df4a3b4` (release workflow corrigé : gtk4-layer-shell
  buildé depuis les sources dans le job `build-overlay` de release.yml)
- **GitHub Release** : publiée, assets = binaire overlay + plugin tgz 1.1.0
- **Validation** : Hyprland 0.56 + Guild Wars 2 (visibilité, click-through,
  focus, speaking show/hide, resize Small=28px/Large=40px mesuré au pixel,
  recovery restart ≈0,35 s, refus seconde instance)
- **Fix majeur** : resize avatar (GtkPicture → GtkImage + pixel_size) —
  GtkPicture se dimensionnait sur la texture téléchargée et ignorait les
  tailles configurées
- **AUR** : PKGBUILD/.SRCINFO 1.1.0 validés par build makepkg local complet
  (build+check+package, binaire 1.1.0, libs résolues, unité embarquée).
  ⚠️ Publication AUR bloquée : secret `AUR_PUSH_TOKEN` absent ET repo privé
  (URL d'archive inaccessible anonymement) → décision propriétaire requise
  (rendre le repo public ou héberger autrement) avant publication AUR.

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

## ✅ Avatar Size Mode Diagnosis — 2026-08-23

- Instrumented runtime probe measured GTK allocations for both modes on live
  keyed rows driven through the REAL Unix socket with REAL 128px async-loaded
  avatar textures: `small` → request 28px, allocated ~33×33;
  `large` → request 40px, allocated 40×40; toggling small→large→small applied
  live without restart or flicker.
- Conclusion: current working-tree source is correct at every boundary
  (plugin serialization shape, socket protocol incl. `"large"`, serde settings
  application, config px mapping, GtkPicture can-shrink allocation).
- Historical failure attributed to a stale deployed overlay binary built
  before commit 91105a2: serde silently ignores unknown JSON fields, so such a
  binary accepts new settings messages and applies position/name/user modes
  live while dropping `avatar_size_mode` — exactly the observed symptom.
- Resolution: rebuild/restart the deployed overlay from this tree when a
  Vesktop environment is available. Added regression tests pinning
  `avatar_size_px` (28/40), `apply_overlay_settings` propagation, and wire
  parsing of `"avatar_size_mode":"large"`.
- Rust fmt/clippy/test/release build, plugin vitest/eslint, and the pinned
  Vencord integration build (`ef29bbeb`, discovery greps) all pass.
- **Human runtime validation PASS (2026-08-23)**: with the freshly built
  overlay, Small → Large → Small visibly resizes avatars live in a real voice
  session. Avatar size mode bug closed.

## ✅ Phase 2 — Game-Ready Hardening — 2026-08-23

- **C1 click-through**: `gdk_surface_set_input_region` (GTK 4.22 / gdk4 0.11)
  with an empty cairo region, applied on every window map; keyboard mode
  stays `None`. Runtime log confirms the region applies on Hyprland; human
  click-through confirmation pending.
- **C2 AUR packaging**: PKGBUILD fixed (`gtk4-layer-shell` in
  depends+makedepends, `pkg-config`/`libadwaita` makedepends,
  `options=(!lto)` because makepkg's gcc `-flto` breaks rust-lld linking of
  ring objects), pkgver 1.1.0, unit file installed to
  `/usr/lib/systemd/user`. Full `makepkg -f --nodeps` build+check pass on a
  local tarball of this tree; packaged binary resolves all shared libs.
- **C3 autostart**: shipped `systemd --user` unit (Restart=always, 2s);
  socket bind now happens before GTK init and a live duplicate instance is
  refused with exit code 1; GTK app made `NON_UNIQUE` so remote activation
  cannot bypass socket ownership; empty overlay window is presented at start
  so the service stays alive before Vesktop connects.
- **C4 reconnect**: plugin backoff capped at 2s (no more 30s pause phase);
  main-process `ResendCache` replays the latest settings + snapshot right
  after every successful handshake, so an overlay restart repopulates without
  any voice activity. Pure logic covered by vitest (`resendCache.test.ts`).
- **C5/C6 avatars**: settings updates no longer reset rows (keyed rows update
  in place); bounded FIFO cache (128 entries) of decoded RGBA keyed by URL;
  one shared single-worker Tokio runtime replaces thread+runtime-per-fetch.
  Measured: 2 HTTP fetches total across snapshot + 3 settings toggles
  (previously refetched on every toggle).
- Gates: cargo fmt/clippy/test(20)/build --release --locked, plugin vitest
  (14)/eslint, Vencord pinned-revision build with discovery greps — all green.

### Phase 2 validation status (test-candidate checkpoint)

- Game-Ready Hardening implementation: **COMPLETE**.
- Automated validation: **PASS** (all authoritative gates green).
- Desktop/Hyprland technical validation: **PASS where actually proven**
  (input region applied on mapped layer surface; single-instance refusal;
  stale socket replacement; 2 avatar fetches across settings toggles).
- Real in-game validation: **PENDING** (gaming PC).
- Pointer click-through over GW2/Dota 2: **PENDING HUMAN VALIDATION**.
- Fullscreen/borderless visibility over games: **PENDING HUMAN VALIDATION**.
- The commit/push of this state is a deployment checkpoint, NOT final
  acceptance; no release or version tag until the game test passes.

---

*Projet livré — v1.0.0 — 2026-08-16*
*Bootstrap COMPLETE — 2026-08-21*

---

## Dependabot triage — 2026-08-25

Triage of the 15 open Dependabot PRs (all major/digest updates that had piled
up outside the minor/patch groups, capped at 5 per ecosystem).

Merged (CI-proven): thiserror 2.0, dirs 6.0, image 0.25, toml 1.1 (cargo);
vitest 4.1 (plugin dev); grouped ts-eslint 6.21 pair; codeql-action v3.37.8
(init+analyze in sync).

Closed: eslint 10 / @typescript-eslint 8 / TS 7 / @types/node 26 (npm majors
need a deliberate flat-config + toolchain migration; majors now suppressed in
dependabot.yml); gio/glib solo bumps (gtk-rs core must move in lockstep —
solo bumps duplicate glib stacks in Cargo.lock); release-path action majors
(setup-node/cache/download-artifact/gh-release) deferred to next release
window since PR CI cannot exercise tag-only workflows.

Config (.github/dependabot.yml): npm + github-actions majors ignored via
wildcard semver-major rule; cargo majors still individual but capped at
open-pull-requests-limit 3; minor/patch groups unchanged. Security updates
unaffected (never grouped).

Incidental CI repair (pre-existing breakage, no product change):
release.yml never loaded on GitHub — duplicate `permissions:` key and
`env` context used in job-level `if` (unavailable there); AUR job also read
an undefined `steps.version` output (would have published empty pkgver).
Fixed by step-level gating + version step ordering. actionlint clean.

Post-triage state: full CI + CodeQL green on main; dependabot now emits one
grouped minor/patch PR per ecosystem per month.
