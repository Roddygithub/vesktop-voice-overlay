# Implementation Plan — Vesktop Voice Overlay

> Historical planning document. Checkboxes and acceptance criteria below are
> goals, not evidence of current implementation. Use `README.md`,
> `memory-bank/architecture.md`, and the validation results for current truth.
> In particular, Vencord does not directly install this project's npm `.tgz`;
> the plugin currently requires a custom source build.

## Vue d'Ensemble Phases

| Phase | Focus | Durée Est. | Livrable |
|-------|-------|------------|----------|
| **P0** | Fondations & Protocole | 3-4 jours | Socket protocol v1, types partagés, CI base |
| **P1** | Vencord Plugin Core | 4-5 jours | Plugin fonctionnel, voice state → socket |
| **P2** | Overlay GTK4 Core | 5-6 jours | Overlay layer-shell, socket server, UI basique |
| **P3** | Intégration & Polish | 3-4 jours | E2E fonctionnel, reconnexion, config, thème |
| **P4** | Distribution Pipeline | 2-3 jours | CI/CD release, AUR PKGBUILD, Vencord manifest |
| **P5** | Tests & Release v1.0.0 | 2-3 jours | E2E complet, doc, tag v1.0.0, publish |

**Total : ~19-25 jours** (étalable sur 6-8 semaines temps partiel)

---

## Phase 0 — Fondations & Protocole (P0)

### Objectif
Définir le protocole socket v1, types partagés TypeScript/Rust, setup monorepo + CI base.

### Tâches

| ID | Tâche | Description | Critères Acceptation |
|----|-------|-------------|----------------------|
| P0.1 | **Monorepo structure** | Créer `plugin/`, `overlay/`, `packaging/`, `.github/workflows/` | `cargo check` + `npm install` passent |
| P0.2 | **Protocole v1 types** | `protocol.ts` (TS) + `protocol.rs` (Rust) identiques | Snapshot sérialisé TS → désérialisé Rust OK |
| P0.3 | **Socket transport spec** | Documenter header, JSON lines, credentials passing, reconnection | Spec écrite dans `docs/protocol.md` |
| P0.4 | **CI base** | `.github/workflows/ci.yml` : lint, test, build check | PR → CI verte |
| P0.5 | **Git hooks** | `lefthook` ou `husky` : format, lint pre-commit | Commit non-formaté rejeté |
| P0.6 | **Version embedding** | `overlay/build.rs` : lit `git describe --tags` → `VERSION` const | Binary `--version` affiche tag |

### Définition de Done P0
- [ ] Monorepo complet, `cargo build` + `npm run build` passent
- [ ] Test protocole : TS serialize → Rust deserialize round-trip OK
- [ ] CI verte sur main
- [ ] `cargo build --release` produit binaire avec version

---

## Phase 1 — Vencord Plugin Core (P1)

### Objectif
Plugin Vencord qui lit voice state, crée snapshots, envoie via Unix socket.

### Tâches

| ID | Tâche | Description | Critères Acceptation |
|----|-------|-------------|----------------------|
| P1.1 | **Vencord manifest** | `manifest.json` : name, version, description, author, entry point | Plugin reconnu par Vesktop |
| P1.2 | **Voice state wrapper** | `voiceState.ts` : hook `useVoiceState`, extraction participants, self | Types stricts, gère canaux vocaux multiples |
| P1.3 | **Socket client** | `socket.ts` : connect `$XDG_RUNTIME_DIR/vesktop-voice-overlay.sock`, reconnection backoff | Connecte à overlay, retry exponentiel |
| P1.4 | **Snapshot builder** | `snapshot.ts` : construit v1 snapshot depuis voice state | JSON valide selon schema v1 |
| P1.5 | **Entry point** | `index.ts` : init socket, écoute voice state changes, envoie snapshots | Snapshots envoyés à chaque changement |
| P1.6 | **Tests unitaires** | Vitest : protocol serialization, socket logic, voice state mapping | `npm test` → 100% pass |
| P1.7 | **Build & pack** | `npm pack` → `.tgz` installable via Vesktop | `.tgz` valide, manifest correct |

### Définition de Done P1
- [ ] Plugin s'installe dans Vesktop (Settings → Plugins → Install from file)
- [ ] Voice join → snapshot envoyé sur socket
- [ ] Speaking start/stop → snapshots temps réel
- [ ] Reconnection auto si overlay restart
- [ ] `npm pack` produit artefact release-ready

---

## Phase 2 — Overlay GTK4 Core (P2)

### Objectif
Overlay natif Wayland (layer-shell) qui reçoit snapshots et affiche participants.

### Tâches

| ID | Tâche | Description | Critères Acceptation |
|----|-------|-------------|----------------------|
| P2.1 | **Layer-shell surface** | `layer_shell.rs` : `zwlr_layer_shell_v1`, click-through, anchor/position | Surface visible, click-through, position configurable |
| P2.2 | **Socket server** | `socket_server.rs` : Unix listener `$XDG_RUNTIME_DIR/...sock`, async tokio | Accepte connexions, valide UID peercred |
| P2.3 | **Protocol deserialization** | `protocol.rs` : parse JSON lines v1, version header, error handling | Snapshots v1 parsés correctement |
| P2.4 | **UI - Participant list** | `ui/participant_list.rs` : GTK4 ListBox, avatar + nom + speaking indicator | Liste mise à jour temps réel |
| P2.5 | **UI - Avatar widget** | `ui/avatar.rs` : Async image load (gio), cache, fallback | Avatars Discord chargées, cache disque |
| P2.6 | **UI - Speaking indicator** | `ui/speaking_indicator.rs` : Pulse animation CSS (keyframes) | Anneau pulsant quand `speaking=true` |
| P2.7 | **Config system** | `config.rs` : TOML `~/.config/.../config.toml`, position, thème, max participants | Config persistée, hot-reload optionnel |
| P2.8 | **Lifecycle management** | `lifecycle.rs` : Socket disconnect → hide, reconnect → show, Vesktop detection | Overlay hide/show auto selon socket state |
| P2.9 | **Tests unitaires** | `cargo test` : protocol deserialize, config, socket logic | `cargo test` passe |
| P2.10 | **Build release** | `cargo build --release --locked` | Binaire optimisé, taille raisonnable |

### Définition de Done P2
- [ ] `cargo run` lance overlay, socket créé
- [ ] Plugin connecte → overlay affiche participants
- [ ] Speaking → pulse animation visible
- [ ] Click-through confirmé (jeu plein écran)
- [ ] Config position/thème persistée
- [ ] Reconnexion auto si plugin restart

---

## Phase 3 — Intégration & Polish (P3)

### Objectif
E2E complet, robustesse, UX, configuration.

### Tâches

| ID | Tâche | Description | Critères Acceptation |
|----|-------|-------------|----------------------|
| P3.1 | **E2E Integration** | Script test : lancer Vesktop + overlay + plugin, join voice, speak | Overlay apparaît, speaking détecté, leave → hide |
| P3.2 | **Reconnexion robuste** | Kill overlay → restart → plugin reconnecte auto | < 2s reconnexion, pas de snapshot perdu |
| P3.3 | **Vesktop lifecycle** | Quitter Vesktop → overlay hide, relancer → show | Détection propre via socket state |
| P3.4 | **Fullscreen XWayland** | Test jeu plein écran (Hyprland) : overlay visible, click-through | Overlay au-dessus, pas d'interférence souris |
| P3.5 | **Multi-compositeurs** | Test Hyprland, sway, niri : layer-shell behavior | Fonctionne sur tous wlroots |
| P3.6 | **Config UI** | Fichier TOML complet : position, thème, avatar size, max participants | Config complète, valeurs par défaut sensées |
| P3.7 | **Thème adaptatif** | CSS suit GTK theme (light/dark), speaking pulse color configurable | Auto light/dark, pulse visible |
| P3.8 | **Accessibilité** | Tooltips, keyboard nav (même si click-through), screen reader labels | a11y basique |
| P3.9 | **Performance** | CPU < 1% idle, < 5% actif, RAM < 50MB | `htop` / `cargo flamegraph` |
| P3.10 | **Logs & Debug** | `RUST_LOG=debug`, structured logging, socket traffic optionnel | Debug facile sans spam |

### Définition de Done P3
- [ ] E2E complet : join → speak → leave → overlay suit
- [ ] Robuste : restart Vesktop/overlay/plugin → récupération auto
- [ ] Fullscreen jeu : overlay visible, click-through
- [ ] Config complète persistée
- [ ] Perf dans cibles

---

## Phase 4 — Distribution Pipeline (P4)

### Objectif
CI/CD release complet : build artefacts, GitHub Release, AUR publish, Vencord manifest.

### Tâches

| ID | Tâche | Description | Critères Acceptation |
|----|-------|-------------|----------------------|
| P4.1 | **Release workflow** | `.github/workflows/release.yml` : tag `v*` → build + release assets | `git tag v1.0.0 && git push --tags` → Release GitHub |
| P4.2 | **Build overlay release** | `cargo build --release --locked` → binary `vesktop-voice-overlay` | Binaire stripped, `--version` affiche tag |
| P4.3 | **Build plugin package** | `cd plugin && npm ci && npm pack` → `.tgz` | Artefact `vesktop-voice-overlay-plugin-<version>.tgz` |
| P4.4 | **GitHub Release** | Assets : binary + plugin.tgz + checksums + release notes | Release page complète |
| P4.5 | **AUR PKGBUILD** | `packaging/aur/PKGBUILD` : build from source, deps, install | `makepkg -si` installe overlay |
| P4.6 | **AUR Publish** | Script CI : update PKGBUILD pkgver/sha256 → push to `aur.archlinux.org` | `yay -S vesktop-voice-overlay` fonctionne |
| P4.7 | **Vencord Manifest** | `plugin/manifest.json` : name, version, description, author, entry | Plugin installable via Vesktop UI |
| P4.8 | **Vencord Store Submit** | Doc/PR vers `Vendicated/Vencord` ou `Vencord/plugins` | Plugin dans store officiel |
| P4.9 | **Checksums & Signing** | SHA256SUMS + cosign/GPG signing des artefacts | Vérifiables par utilisateurs |
| P4.10 | **Install Script** | `install.sh` optionnel : détecte distro, installe overlay + guide plugin | `curl -fsSL ... | bash` optionnel |

### Définition de Done P4
- [ ] `git tag v1.0.0 && git push --tags` → Release GitHub complète
- [ ] `yay -S vesktop-voice-overlay` installe overlay
- [ ] Plugin `.tgz` installable via Vesktop UI
- [ ] AUR package à jour
- [ ] Artefacts signés/vérifiables

---

## Phase 5 — Tests & Release v1.0.0 (P5)

### Objectif
Validation finale, documentation, release publique.

### Tâches

| ID | Tâche | Description | Critères Acceptation |
|----|-------|-------------|----------------------|
| P5.1 | **E2E Complet Automatisé** | Script CI : Vesktop headless + overlay + plugin → join → speak → leave | Pipeline CI verte |
| P5.2 | **Matrix Compatibilité** | Test Hyprland, sway, niri, labwc (VM ou CI matrix) | Tous wlroots OK |
| P5.3 | **Security Audit** | Review code : pas de token, pas réseau, socket user-only, pas `unsafe` Rust critique | Audit passé |
| P5.4 | **Documentation Complète** | README : install, config, troubleshooting, architecture, contributing | Docs utilisateur + dev complètes |
| P5.5 | **CHANGELOG** | `CHANGELOG.md` depuis v0.1.0 → v1.0.0 | Historique clair |
| P5.6 | **Release v1.0.0** | Tag `v1.0.0` → pipeline complète → artefacts publiés | Release publique |
| P5.7 | **Annonce & Feedback** | Post Reddit r/archlinux, r/linux, Discord Vencord, GitHub Discussions | Premiers utilisateurs |
| P5.8 | **Bug Triage Post-Release** | Labels GitHub, milestones v1.0.1, v1.1.0 | Processus maintenance |

### Définition de Done P5
- [ ] v1.0.0 taggée, Release GitHub publiée
- [ ] AUR package à jour
- [ ] Plugin dans Vencord Store (ou installable via URL)
- [ ] Doc complète
- [ ] Premiers retours utilisateurs collectés

---

## Dépendances & Ordre

```
P0.1 → P0.2 → P0.3 → P0.4 → P0.5 → P0.6
                              ↓
P1.1 → P1.2 → P1.3 → P1.4 → P1.5 → P1.6 → P1.7
                              ↓
P2.1 → P2.2 → P2.3 → P2.4 → P2.5 → P2.6 → P2.7 → P2.8 → P2.9 → P2.10
                              ↓
P3.1 → P3.2 → P3.3 → P3.4 → P3.5 → P3.6 → P3.7 → P3.8 → P3.9 → P3.10
                              ↓
P4.1 → P4.2 → P4.3 → P4.4 → P4.5 → P4.6 → P4.7 → P4.8 → P4.9 → P4.10
                              ↓
P5.1 → P5.2 → P5.3 → P5.4 → P5.5 → P5.6 → P5.7 → P5.8
```

---

## Jalons Clés (Milestones)

| Jalon | Critère | Date Cible |
|-------|---------|------------|
| **M1 - Protocol Ready** | P0 Done : protocole v1 figé, CI verte | S+1 |
| **M2 - Plugin Works** | P1 Done : plugin envoie snapshots | S+2 |
| **M3 - Overlay Works** | P2 Done : overlay affiche participants | S+3 |
| **M4 - E2E Works** | P3 Done : join/speak/leave complet | S+5 |
| **M5 - Dist Ready** | P4 Done : AUR + Vencord installables | S+7 |
| **M6 - v1.0.0 Released** | P5 Done : release publique | S+8 |

---

## Risques & Plans B

| Risque | Probabilité | Impact | Plan B |
|--------|-------------|--------|--------|
| API Vencord change (breaking) | Élevée | Bloquant P1 | Pin manifest version, wrapper adaptable, tests CI nightly |
| Layer-shell bug sur certains wlroots | Moyenne | Dégradé P2 | Fallback position, rapport upstream, workaround config |
| Socket permissions (SELinux/AppArmor) | Faible | Bloquant P2 | Doc troubleshooting, fallback abstract socket |
| `npm pack` artefacts trop gros | Faible | Mineur | Optimize bundle (esbuild), exclude dev deps |
| AUR review lent | Moyenne | Retard P4 | Pre-submit review, community co-maintainer |
| Vencord store rejection | Faible | Retard P4 | Distribution via GitHub Release URL (fallback) |

---

## Commandes Validation Rapide (Dev Loop)

```bash
# Build tout
cd ~/Projets/vesktop-voice-overlay
cargo build --release --manifest-path overlay/Cargo.toml
cd plugin && npm ci && npm run build && npm pack

# Test protocole round-trip
cd plugin && npm test
cd ../overlay && cargo test

# Lancer overlay seul (debug)
RUST_LOG=debug ./overlay/target/release/vesktop-voice-overlay

# Test socket manuel
echo 'VESKTOP_VOICE_OVERLAY/1.0
{"version":1,"timestamp":123,"self":{"userId":"1","username":"Test","avatarUrl":"","mute":false,"deaf":false,"speaking":true},"participants":[]}' | nc -U $XDG_RUNTIME_DIR/vesktop-voice-overlay.sock

# Full dev loop (nécessite Vesktop + Vencord)
# 1. Terminal 1: overlay
# 2. Terminal 2: Vesktop avec plugin installé
# 3. Rejoindre vocal → vérifier overlay
```
