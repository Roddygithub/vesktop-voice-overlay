# Progress — Vesktop Voice Overlay

## Statut Global

| Phase | Statut | Progression | Début | Fin Estimée | Notes |
|-------|--------|-------------|-------|-------------|-------|
| **P0 - Fondations & Protocole** | 🔴 Non commencé | 0% | J+0 | J+3 | Structure monorepo, spec protocole v1, types TS/Rust |
| **P1 - Vencord Plugin Core** | 🔴 Non commencé | 0% | J+3 | J+5 | Extraction voice state, socket client, build |
| **P2 - Overlay GTK4 Core** | 🔴 Non commencé | 0% | J+5 | J+8 | Surface layer-shell, socket server, UI widgets |
| **P3 - Intégration & Polish** | 🔴 Non commencé | 0% | J+8 | J+11 | E2E, reconnexion, click-through jeu, thème |
| **P4 - Distribution Pipeline** | 🔴 Non commencé | 0% | J+11 | J+14 | CI/CD GitHub release, PKGBUILD AUR, Vencord pack |
| **P5 - Tests & Release v1.0.0** | 🔴 Non commencé | 0% | J+14 | J+15 | Validation E2E, audit, doc complète, release |

**Progression Globale** : 5% (Initialisation de la structure, planification & architecture complétées)

---

## Journal de Bord

### 2026-08-15 — Cadrage & Planification Architecture
- ✅ Identification du problème et des personas
- ✅ Alignement technique : Vencord Plugin (TypeScript) ↔ Unix Socket ↔ Overlay (Rust + GTK4 / layer-shell)
- ✅ **Décision de Distribution** : Monorepo de développement, mais distribution séparée (AUR pour l'binaire overlay, Vencord Plugin Store pour l'extension Vesktop).
- ✅ Initialisation de la Memory Bank :
  - `prd.md` : Besoins, portée MVP, critères d'acceptation, plan de distribution.
  - `tech-stack.md` : Pile technique complète (Rust, GTK4, Tokio, Serde, TypeScript, Vencord API).
  - `architecture.md` : Diagrammes d'architecture, spécifications du protocole v1, configuration, structure monorepo.
  - `implementation-plan.md` : Planification détaillée en 6 phases (P0 à P5) avec jalons et gestion des risques.
  - `progress.md` : Suivi global et journal de bord.

---

## Prochaines Actions Immédiates

### Phase 0 — Fondations & Protocole

1. **Création de la structure du monorepo** :
   ```bash
   mkdir -p plugin/src overlay/src packaging/aur .github/workflows
   ```

2. **Écriture de la spécification de protocole v1** dans `docs/protocol.md` (types TS/Rust).

3. **Setup du Tooling BMad / Vibe Coding** :
   ```bash
   ./scripts/bootstrap-bmad.sh
   ./scripts/bootstrap-vibe-coding.sh
   ```

---

## Dépendances Externes à Valider

| Dépendance | Statut | Action Requise |
|------------|--------|----------------|
| `rustc`/`cargo` installés | ❓ Inconnu | `cargo --version` |
| `node`/`npm` installés | ❓ Inconnu | `node --version && npm --version` |
| `gtk4` / `libadwaita` de dev | ❓ Inconnu | `pkg-config --modversion gtk4` |
| Vesktop + Vencord | ❓ Inconnu | Lancer Vesktop et vérifier l'injection Vencord |

---

## Décisions Techniques Enregistrées

| Date | Décision | Raison | Alternative Rejetée |
|------|----------|--------|---------------------|
| 2026-08-15 | Monorepo de dev, Dist séparée | Coordonner les releases tout en respectant les standards de packaging Linux (AUR) et Vencord (Store/tgz) | Mono-paquet AUR global (impossible de gérer proprement les plugins Vencord système), ou repos complètement séparés (perte de cohésion de release) |
| 2026-08-15 | Rust + GTK4 / layer-shell | Performance, Wayland natif, click-through fiable, faible empreinte RAM/CPU | Electron overlay (trop lourd, instable sur Wayland), Python+GTK (lenteur, distribution binaire complexe) |
| 2026-08-15 | Unix Socket local | Performance, isolation par utilisateur, pas d'exposition réseau | WebSocket / TCP (surcoût, risque sécurité) |

---

## Prochaine Session - Checklist

- [ ] Structurer les dossiers `plugin/` et `overlay/`
- [ ] Valider l'environnement de build (Node + Rust)
- [ ] Écrire les structures de données de protocole v1 partagées
- [ ] Mettre en place la CI de base dans GitHub Actions
- [ ] Mettre à jour `progress.md`

---

*Dernière MAJ : 2026-08-15 — Session initialisation + planification architecture*