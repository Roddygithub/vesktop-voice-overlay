# Progress — Discord Voice Overlay

## Statut Global

## Universal Installer Candidate — 2026-08-31

### Clean-target acceptance — blocked

- A real alternate native Discord profile was proven representative for
  per-profile application resources, but cannot exercise this candidate's user
  service path because the running systemd user manager is bound to the normal
  XDG configuration directory.
- With explicit approval, a rollback bundle was created at
  `/home/roddy/.cache/dvo-clean-target-rollback`, containing only Discord
  application resources, the Vesktop state file, overlay binary, and legacy
  service files. It was verified byte-for-byte before mutation.
- The official Vencord CLI restored the live Discord target cleanly. The real
  installer then downloaded and checksum-verified the v1.2.1 overlay, cloned
  Vencord at revision `ef29bbeb6119cfb53d1273ed78147bcc97d91261`, built the
  plugin, and created the managed service.
- Installation blocked on two genuine defects: `pnpm inject` invokes the
  interactive official selector despite `--yes` and `DISCORD_USER_DATA_DIR`,
  and `systemd-escape --path` generated a non-absolute `ExecStart`, causing
  service auto-restart. Discord injection and installer state publication did
  not complete.
- The partial managed tree was removed through the candidate ownership guard.
  Original Discord resources, Vesktop state, overlay, service, and drop-in were
  restored byte-for-byte; the legacy service is enabled and running. No human
  voice test was attempted.

### Corrected installer retest — blocked by clean-user plugin default

- The pinned CLI syntax was verified from its help output as
  `-install`/`-uninstall` with `-location <Discord app directory>`. The
  installer now invokes that official wrapper directly for install, rollback,
  client switching, and uninstall; no interactive selector appeared during the
  corrected live install.
- Service generation now quotes an absolute executable path for systemd
  `ExecStart`. The generated live unit passed `systemd-analyze verify`, started
  successfully, remained running with `NRestarts=0`, and exposed the overlay
  socket.
- Real corrected install completed checksum verification, pinned Vencord clone
  and build, official Discord injection, state publication, service startup,
  native Discord startup, and status/doctor checks.
- Clean-user runtime validation found the pinned Vencord default
  `VesktopVoiceOverlay.enabled=false`; the plugin therefore did not start or
  connect to the overlay socket. No manual settings edit was made because it is
  outside the requested two fixes.
- The corrected managed uninstall path was exercised after the runtime block;
  the official uninject path still requires the same explicit target and the
  original Discord resources, Vesktop state, overlay, service, and drop-in were
  restored byte-for-byte. The legacy service is enabled and running.

### Managed plugin enablement — automated validation passed

- Pinned Vencord source confirms renderer settings are JSON at
  `<XDG_CONFIG_HOME>/Vencord/settings/settings.json`; plugin state is keyed by
  the preserved internal name `VesktopVoiceOverlay`, defaults to disabled, and
  has no installer-facing CLI setter.
- The installer now performs a structural, atomic update of only
  `plugins.VesktopVoiceOverlay.enabled`, preserves unknown/unrelated settings,
  backs up attributable prior content, restores it conservatively on uninstall,
  and rolls it back on failed unpublished installs.
- Fixture coverage passes for disabled/missing/malformed settings, unrelated
  enabled/disabled plugins, custom plugin settings, repair, idempotence,
  uninstall preservation, foreign integration refusal, explicit Discord CLI
  targeting, and absolute systemd `ExecStart` validation.
- Full Rust and plugin gates pass again; ShellCheck remains unavailable locally.
- The refreshed rollback bundle includes the current Vencord settings file and
  matches the current known-good Discord, Vesktop, overlay, and service files
  byte-for-byte before live mutation.
- Approved clean-target live install now completed with automatic plugin
  enablement. The actual settings file shows `enabled: true` while all other
  settings remain semantically identical to the rollback copy; ownership
  metadata matches the managed hash.
- Native Discord was started against the patched target. Its renderer log shows
  `Starting plugin VesktopVoiceOverlay`; the overlay journal shows a UID-valid
  client connection, settings update, and clear message. The Unix socket is
  established and the service remains active with `NRestarts=0`.
- Human voice acceptance passed: the overlay appeared correctly, self speaking
  state worked, and another participant's speaking state worked.
- Deferred UX follow-up: the Vencord Plugins UI displays the internal plugin
  name `VesktopVoiceOverlay` instead of the public product name `Discord Voice
  Overlay`. Do not change this during the current acceptance or rollback
  lifecycle; address it separately as a naming/UX task.
- Lifecycle repair and update both completed idempotently while installed.
  Uninstall completed through the official explicit-target uninject path and
  removed the managed tree, state, and service. Vencord had persisted plugin
  settings during runtime (including user-facing plugin options), so the
  conservative ownership guard correctly warned and preserved the changed
  settings rather than overwriting them.
- Clean reinstall completed after uninstall, including a fresh pinned Vencord
  build, automatic reuse of the already-enabled plugin setting, official
  Discord injection, and successful final uninstall. The original legacy
  service was restored and is active with zero restarts; Discord resources,
  Vesktop state, overlay binary, service unit, and managed paths passed final
  restoration checks.
- Final automated gates pass: installer fixtures, Rust format/Clippy/tests,
  plugin lint/tests (`25/25`), and npm audit (`0` vulnerabilities). The plugin
  test was rerun with `TMPDIR` on the home filesystem because the machine's
  unrelated `/tmp` tmpfs was full.
- Final-state caveat: shared Vencord settings remain changed and
  `VesktopVoiceOverlay.enabled` remains true because runtime/plugin settings
  changed after installation; the installer intentionally preserved them under
  its conservative ownership rule. The original settings copy remains in the
  approved rollback bundle for an explicit follow-up decision.

### v1.3.0 release preparation — local gates passed

- Release metadata is aligned at `1.3.0` for the installer, overlay binary,
  plugin package, lockfiles, AUR template metadata, and installer fixture.
  Historical v1.2.1 records and compatibility-sensitive identifiers remain
  unchanged.
- Changelog and installer documentation now describe the universal native Arch
  manager, supported commands, ownership behavior, limitations, and deferred
  AUR publication. The Vencord UI naming follow-up remains documentation-only.
- Final local checks pass: `bash -n`, all installer fixtures, Rust fmt/Clippy
  with warnings denied, `39/39` Rust tests, release and release-debug builds,
  binary version `1.3.0`, plugin `npm ci`/lint/tests (`25/25`)/audit (`0`),
  package dry-run, and generated systemd verification.
- ShellCheck is unavailable locally and remains a required GitHub CI gate. The
  release workflow's pinned Vencord build/discovery validation remains required
  on the tag workflow; the same pinned build and plugin discovery were already
  proven during live acceptance.
- The first pushed v1.3.0 candidate CI run executed the new installer gate and
  failed only on actionable ShellCheck findings: an unused loop variable, an
  ambiguous `&&`/`||` conditional, and test-harness source/assignment/quoting
  analysis warnings. These were fixed directly without blanket suppressions;
  local syntax and all installer fixtures pass again.
- The follow-up CI run reduced the remaining ShellCheck failure to an unused
  local declaration and dynamic test-source analysis. The declaration is now
  removed and CI runs ShellCheck with `-x` so the explicitly documented
  `install.sh` test source is followed rather than suppressed.
- The next CI run confirmed source following and reduced the last failure to
  the directive's relative path. It now points to the checkout-root
  `install.sh`, matching the CI working directory.
- The resulting CI run failed in the installer fixture rather than the
  installer: its mock `node` wrapper delegated to `/usr/bin/node`, which is
  not the setup-node location on GitHub runners. The fixture now captures the
  runner-provided Node path before prepending its mock directory to `PATH`;
  local syntax and all installer cases pass with the portable delegation.

- Reconciled release baseline `v1.2.1` at
  `61e3e2ae4135acd9eff55bf2a3a15fb4677d3f33`; the existing tag points at the
  release commit. Installer candidate changes remain uncommitted by request.
- Native Arch packages detected locally: Vesktop `1.6.7-1` and Discord
  `1:1.0.155-1`.
- The local Vesktop `state.json` points at an existing custom Vencord checkout;
  the local Discord target contains an existing injected POC. Both are foreign
  integrations for installer purposes and must not be silently replaced.
- The installer design uses a user-owned managed Vencord checkout, verified
  release artifacts, a user systemd unit, explicit state, and safe client
  selection. No live client mutation has been performed for this candidate.
- Installer fixture validation passes after lifecycle, idempotence, client
  selection, foreign integration, checksum, failure rollback, state, and
  recursive-deletion guard coverage. `bash -n` also passes; local ShellCheck is
  unavailable and is configured in CI.
- Existing project validation still passes: Rust format check, Clippy with
  warnings denied, `39/39` Rust tests; plugin lint, `25/25` tests, and npm audit
  with zero vulnerabilities.
- Live `status` and `doctor` were run without mutation. Both native clients and
  the existing active user service were detected; no installer-owned state was
  adopted.
- Live Discord install classification was exercised with the real candidate;
  it refused the existing foreign `_app.asar` injection before any mutation.
  `update` refused the ambiguous no-state dual-client case and `repair` refused
  the absent installer state. Post-check hashes confirm Discord resources,
  Vesktop state, overlay, service files, and repository candidate files were
  unchanged. Human voice validation is deferred because forcing adoption would
  require manually destroying the known-good POC.

## v1.2.1 Release Candidate Validation — 2026-08-31

- Public branding is now **Discord Voice Overlay**; the persisted plugin name,
  native bridge lookup, protocol header, binary, service, socket, and config
  identifiers remain unchanged for compatibility.
- Version metadata is aligned at `1.2.1` across the plugin, Cargo package, AUR
  template, generated `.SRCINFO`, and release changelog.
- Plugin validation passed: `npm ci`, lint, `25/25` tests, audit with zero
  vulnerabilities, and npm package dry-run for the `1.2.1` source bundle.
- Overlay validation passed: format check, clippy with warnings denied,
  `39/39` tests, release build, and release-debug build.
- Pinned Vencord integration passed at revision
  `ef29bbeb6119cfb53d1273ed78147bcc97d91261`; both desktop bundles contain the
  `VesktopVoiceOverlay` plugin identifier.

## Custom position diagnosis and fix — 2026-08-31

- **Root cause**: Custom X/Y changes were serialized and received correctly,
  but the active `position` remained `top-right`; Rust therefore correctly
  ignored the custom margins. This was a shared plugin-settings UX/state issue,
  not Discord, IPC, serde, GTK, or compositor failure.
- **Evidence**: live logs recorded `position=top-right, custom_x=800,
  custom_y=300`; forcing `position=custom` produced a Hyprland layer at
  `x=800`; the observed `y=326` includes Hyprland's 26px exclusive top bar.
- **Fix**: changing either Custom X or Custom Y now activates
  `position=custom`; values changed while already custom still send immediately.
- **Plugin regression**: the shared `positionForCustomCoordinateChange` policy
  is covered by the plugin suite; both coordinate settings use the same handler.
- **Preset regression**: the existing `center` branch incorrectly anchored all
  four edges, placing the natural-size surface at the usable top-left. Leaving
  it unanchored restores compositor-centered placement.
- **Validation**: Rust fmt/clippy/tests and release/release-debug builds pass;
  plugin tests `25/25`, lint, and npm audit pass; pinned Discord-target
  Vencord build passes.
- **Layer-shell probe**: all six modes now produced distinct expected Hyprland
  geometries; custom `800/300` mapped at `800/326` with the 26px top bar.
- **Human validation PASS**: Custom X/Y movement, custom position persistence
  after overlay restart/reconnect, and all previously requested Discord Desktop
  runtime checks passed in the rebuilt POC.

## v1.2.0 human validation — 2026-08-31

- **PASS**: real multi-user Discord voice call completed with the current
  v1.2.0 candidate deployment.
- **PASS**: self/other rows, speaking rings, mute/deaf badges, click-through
  over the game, and live participant avatar refresh without reverting.
- **PASS**: overlay restart replayed settings and participants; Vesktop restart
  reconnected successfully; leaving the voice channel cleared the overlay.
- This closes the final human-only release gate. No source changes were made
  during deployment or validation.

## v1.2.0 release execution — 2026-08-31

- **PASS**: commits `8db0f62` and `996a91f` were pushed to `main`; CI run
  `33369404382` and CodeQL run `33369404207` completed successfully on
  `996a91f3e7c52f1da355db32859c1d3aa1830c97`.
- **PASS**: annotated tag `v1.2.0` was pushed and release workflow
  `33369848885` completed successfully, including overlay/plugin builds and
  GitHub Release creation.
- **PASS**: published overlay and plugin assets were downloaded and both
  entries in `SHA256SUMS` matched locally.
- **AUR**: publication was skipped cleanly because `AUR_SSH_PRIVATE_KEY` is
  not configured. The AUR RPC reports no `vesktop-voice-overlay` package;
  first publication therefore still requires registering the package in the
  AUR and providing a maintainer SSH key. The current workflow can update an
  existing AUR repository but cannot bootstrap the registration itself.

## v1.2.0 final XHIGH review — 2026-08-31

- Independent challenge found unsafe `/tmp` fallback trust, dropped avatar
  fetches beyond eight, stale delayed-hide ordering, authoritative state loss
  under Node backpressure, input-order-dependent duplicate IDs, and broken
  first-publication/checksum/license paths in the tag workflow.
- Minimal corrections now fail closed without `$XDG_RUNTIME_DIR`, queue avatar
  work, cap decoded cache entries to 256px images, coalesce authoritative
  backpressure state, clear only current disconnect state, admit one IPC client,
  and exercise actual GTK row reuse/order/badges/removal in the aggregate test.
- Final validation passes: Rust fmt/clippy, 39 tests, release/release-debug
  builds; plugin clean install, 24 tests, lint, zero-vulnerability audit, and
  seven-file licensed package; pinned Vencord build/discovery; actionlint/YAML,
  flat checksum simulation, AUR metadata/shell, and systemd verification.
- The exact root-container `runuser` command and future tag-triggered GitHub/AUR
  publication remain environment/release-only checks, not locally executed.

## v1.2.0 release-candidate audit — 2026-08-28

- Full source/history/dependency/workflow review found release-blocking defects
  despite green CI: stale UI after voice leave, false deterministic ordering,
  allocation-before-limit in JSONL reads, incorrect multi-client disconnect
  state, unbounded command/socket backpressure paths, unsafe avatar resource
  assumptions, and broken tag/AUR workflow paths.
- Corrective work adds authoritative clear/replay, bounded IPC and avatar I/O,
  live race-safe avatar refresh, real ordering/deduplication tests, same-UID
  connection limits, tag/version checks, checksums, deterministic integration
  inputs, and truthful source-userplugin documentation.
- The npm `.tgz` is confirmed to be a source bundle, not a directly installable
  Vesktop/Vencord plugin. A supported end-user plugin distribution channel
  remains unresolved.
- Automated validation status and human-only gates are recorded in the final
  maintainer report; prior PASS entries below are historical evidence only.
- Interrupted avatar recovery completed: request generations now prevent stale
  `A -> B -> A` completions from mutating current widget state, same-URL loads
  are deduplicated, failures remain retryable, and fetch concurrency is restored
  to the evidence-backed limit of eight. Eleven focused avatar tests and
  all-target Clippy pass; the GTK sizing test executed successfully.
- Final mechanical validation exposed and fixed a GTK test-only SIGSEGV: the
  coredump showed all three GTK tests concurrently inside `gtk_init_check` on
  separate harness threads. They now run as one GTK aggregate test. The default
  parallel suite passes 39/39; the aggregate executed on the active display.
- The HIGH phase reported authoritative Rust, plugin, Vencord, workflow, and
  packaging validation as passing. XHIGH later found release-path gaps that
  syntax and local metadata simulations did not cover; its corrections and
  final validation are recorded above.

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
- **GitHub Release** : publiée, assets = binaire overlay + plugin source tgz 1.1.0
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
- **Vencord Store** : historical claim was incorrect; `.tgz` is source only

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
