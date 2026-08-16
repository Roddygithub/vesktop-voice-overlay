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

*Projet livré — v1.0.0 — 2026-08-16*
