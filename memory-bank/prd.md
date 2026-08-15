# Product Requirements Document — Vesktop Voice Overlay

## 1. Problème

Les utilisateurs de Vesktop (client Discord basé sur Vencord) sur Wayland/Hyprland n'ont pas d'indicateur natif d'activité vocale (qui parle, avatar, état) intégré au desktop. Les solutions existantes :
- Nécessitent un bot/self-bot (violations ToS Discord)
- Sont des overlays X11 via XWayland (pas natif Wayland, pas de click-through)
- Accèdent au token Discord (risque sécurité)

## 2. Utilisateurs & Personas

| Persona | Besoins |
|---------|---------|
| **Gamer Hyprland** | Overlay natif Wayland, click-through, visible en plein écran jeu |
| **Développeur/Privacy** | Pas de token Discord, pas de self-bot, code auditable |
| **Utilisateur Vesktop/Vencord** | Installation simple via Vencord plugin store, pas de config manuelle |

## 3. Portée Initiale (MVP)

### Core Features
- **Vencord Plugin** : Lit le voice state interne (renderer process) → snapshots versionnés
- **Unix Socket** : `$XDG_RUNTIME_DIR/vesktop-voice-overlay.sock` (user-only, credentials passing)
- **Overlay GTK4/layer-shell** : Click-through, avatars + noms + speaking indicator, position configurable

### MVP Scope
- Afficher participants canal vocal actif (avatar, display name, speaking state)
- Click-through layer-shell surface (wlroots: Hyprland, sway, niri, etc.)
- Reconnexion auto si Vesktop ou overlay redémarre
- Validation sur jeu plein écran XWayland (Hyprland)
- Protocole versionné (JSON sur socket) pour compatibilité future

## 4. Critères de Succès

- [ ] Overlay apparaît/disparaît automatiquement selon état vocal
- [ ] Speaking indicator temps réel (< 100ms latence)
- [ ] Fonctionne sur jeu plein écran XWayland (Hyprland)
- [ ] Zéro token Discord lu/stocké/transmis
- [ ] Installation en 2 étapes max (AUR + Vencord plugin)
- [ ] Reconnection < 2s après restart Vesktop/overlay

## 5. Hors Périmètre (v1)

- Historique participants / logs
- Configuration UI avancée (position, thème, taille)
- Support multi-canaux simultanés
- Intégration Noctalia (projet séparé)
- Packaging Flatpak / Snap (post-v1)

## 6. Contraintes

| Contrainte | Détail |
|------------|--------|
| **Sécurité** | Aucun token Discord, aucun self-bot, aucune connexion Gateway |
| **Architecture** | Plugin Vencord (renderer) → Unix socket → Overlay (GTK4/layer-shell) |
| **Distribution** | **Monorepo source** → **Distribution séparée** : AUR (overlay binaire) + Vencord plugin store (plugin TS) |
| **Compatibilité** | Wayland natif (layer-shell), wlroots compositors (Hyprland, sway, niri, etc.) |
| **Licence** | GPL-3.0 (compatibilité Discover Overlay, Vencord, Vesktop) |
| **Versioning** | Un tag Git = plugin + overlay compatibles (protocole socket versionné) |

## 7. Distribution Strategy (Décision Architecturale Majeure)

### Monorepo Source (ce repo)
```
vesktop-voice-overlay/
├── plugin/           # Vencord plugin (TypeScript)
│   ├── src/
│   ├── manifest.json
│   └── package.json
├── overlay/          # Overlay GTK4/layer-shell (Rust)
│   ├── Cargo.toml
│   └── src/
├── packaging/
│   └── aur/          # PKGBUILD pour AUR
└── .github/workflows/
    └── release.yml   # CI: build overlay + plugin zip + AUR package
```

### Distribution Séparée (Pour l'utilisateur final)

| Composant | Canal | Installation Utilisateur |
|-----------|-------|--------------------------|
| **Overlay binaire (Rust/GTK4)** | **AUR** | `yay -S vesktop-voice-overlay` |
| **Vencord Plugin (TypeScript)** | **Vencord Plugin Store** | Vesktop → Settings → Plugins → "Install from Store" ou URL |

### Pourquoi séparé ?
- **AUR** = binaire système géré par pacman (dépendances GTK4, layer-shell, etc.)
- **Vencord Plugin Store** = installation native Vesktop (Settings → Plugins), mises à jour auto, pas de copie manuelle de fichiers
- **Versioning unifié** : Tag `v1.0.0` = plugin + overlay compatibles (protocole socket v1)

## 8. Métriques de Succès (KPIs)

| Métrique | Cible |
|----------|-------|
| Latence speaking indicator | < 100ms |
| Temps installation utilisateur | < 2 min (AUR + Vencord) |
| Taux crash overlay | 0% sur 1 semaine usage continu |
| Reconnexion après restart | < 2s |
| Stars GitHub | 50+ en 3 mois |
| AUR votes | 10+ en 3 mois |