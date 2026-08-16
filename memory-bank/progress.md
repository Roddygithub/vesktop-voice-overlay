# Progress — Vesktop Voice Overlay

## Statut Global

| Phase | Statut | Progression | Début | Fin Estimée | Notes |
|-------|--------|-------------|-------|-------------|-------|
| **P0 - Fondations & Protocole** | 🟢 Terminé | 100% | J+0 | J+3 | Structure monorepo, spec protocole v1, types TS/Rust, CI base |
| **P1 - Vencord Plugin Core** | 🟢 Terminé | 100% | J+3 | J+5 | Plugin complet, build + tests OK |
| **P2 - Overlay GTK4 Core** | 🟢 Terminé | 100% | J+5 | J+8 | Layer-shell, socket server, UI, build + tests OK |
| **P3 - Intégration & Polish** | 🟢 Validé | 90% | J+8 | J+11 | Socket protocol testé sur Wayland (Hyprland) |
| **P4 - Distribution Pipeline** | 🟡 Prêt | 80% | J+11 | J+14 | PKGBUILD AUR, release workflow créés |
| **P5 - Tests & Release v1.0.0** | 🟡 Prêt | 50% | J+14 | J+15 | Tag v1.0.0 prêt à pousser |

**Progression Globale** : 85% (Toutes phases structurelles complètes, prêt pour release)

---

## Journal de Bord

### 2026-08-15 — Cadrage & Planification
- Architecture Vencord Plugin ↔ Unix Socket ↔ Overlay GTK4
- Distribution séparée : AUR (overlay) + Vencord Store (plugin)

### 2026-08-16 — Implémentation complète
- ✅ Monorepo + CI GitHub Actions
- ✅ Protocole v1 (JSON Lines, header versionné)
- ✅ Plugin Vencord : build, tests (6), socket client, reconnexion
- ✅ Overlay Rust : build release, tests (5), layer-shell click-through
- ✅ Socket testé sur Hyprland : connexion + header + snapshot OK
- ✅ AUR PKGBUILD + release workflow
- ✅ CHANGELOG.md

---

## Prêt pour Release v1.0.0

### Checklist finale
- [x] Tests plugin passent (6/6)
- [x] Tests overlay passent (5/5)
- [x] Build release overlay fonctionne
- [x] Build plugin (npm pack) fonctionne
- [x] Socket protocol validé sur Wayland
- [x] CI workflow configuré
- [x] Release workflow configuré (GitHub Release + AUR)
- [x] CHANGELOG.md à jour
- [ ] Tag `v1.0.0` et push

### Commandes pour release
```bash
cd /home/roddy/Projects/vesktop-voice-overlay
git add -A
git commit -m "chore: prepare v1.0.0 release"
git tag v1.0.0
git push origin main --tags
```

### Artefacts générés par CI
| Artefact | Destination |
|----------|-------------|
| `vesktop-voice-overlay` (binaire) | GitHub Release + AUR |
| `vesktop-voice-overlay-plugin-1.0.0.tgz` | GitHub Release + Vencord Store |

---

## Prochaines étapes post-release
1. Publier plugin sur Vencord Store (PR vers Vendicated/Vencord)
2. Tester installation AUR : `yay -S vesktop-voice-overlay`
3. Documentation utilisateur (README déjà complet)
4. Collecter retours utilisateurs

---

*Dernière MAJ : 2026-08-16 — Projet prêt pour v1.0.0*
