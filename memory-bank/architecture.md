# Architecture — Vesktop Voice Overlay

## 1. Vue d'Ensemble Système

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           VESKTOP (Electron + Vencord)                      │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Renderer Process (WebView)                                         │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │  Vencord Plugin (TypeScript)                                │   │   │
│  │  │  - Accède au voice state interne (useVoiceState, etc.)     │   │   │
│  │  │  - Sérialise snapshots versionnés (JSON)                   │   │   │
│  │  │  - Envoie via Unix socket (credentials passing)            │   │   │
│  │  │  - Gère reconnexion auto                                    │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                    Unix Socket (SCM_RIGHTS, user-only)
                    $XDG_RUNTIME_DIR/vesktop-voice-overlay.sock
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        OVERLAY PROCESS (Rust + GTK4)                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  GTK4 + layer-shell (Wayland native)                               │   │
│  │  - Click-through surface (zwlr_layer_surface_v1)                   │   │
│  │  - Reçoit snapshots via socket                                     │   │
│  │  - Rend: avatars, noms, speaking indicator (pulse animation)       │   │
│  │  - Position configurable (coin, centre, custom)                    │   │
│  │  - Gère cycle de vie Vesktop (socket disconnect = hide)            │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 2. Composants Détaillés

### 2.1 Vencord Plugin (`plugin/`)

**Responsabilités** :
- Accès au voice state interne Vesktop via API Vencord (`useVoiceState`, `useCurrentUser`, etc.)
- Détection changements : join/leave/mute/deaf/speaking
- Sérialisation snapshots versionnés (JSON) avec timestamp + version protocole
- Envoi via Unix socket avec `SCM_RIGHTS` (file descriptor passing pour sécurité)
- Gestion reconnexion : exponential backoff, max 5 tentatives, puis pause
- Cycle de vie : se connecte au démarrage Vesktop, nettoie socket à l'arrêt

**Architecture Interne** :
```
plugin/src/
├── index.ts              # Entry point Vencord
├── socket.ts             # Unix socket client (connect, send, reconnect)
├── protocol.ts           # Types TypeScript + serialization (v1, v2...)
├── voiceState.ts         # Wrapper API Vencord voice state
├── snapshot.ts           # Création snapshots (participants, speaking, self)
└── manifest.json         # Vencord manifest (name, version, description, author)
```

**Protocole Socket (v1)** :
```typescript
// Plugin → Overlay
interface Snapshot {
  version: 1;
  timestamp: number;           // Date.now()
  self: {                      // Utilisateur local
    userId: string;
    username: string;
    avatarUrl: string;
    mute: boolean;
    deaf: boolean;
    speaking: boolean;
  };
  participants: Participant[]; // Autres dans le canal
}

interface Participant {
  userId: string;
  username: string;
  avatarUrl: string;
  speaking: boolean;
  volume?: number;             // 0-100 (si dispo)
}
```

### 2.2 Overlay GTK4 (`overlay/`)

**Responsabilités** :
- Crée surface layer-shell (zwlr_layer_surface_v1) avec `keyboard_interactivity=none` (click-through)
- Écoute Unix socket, désérialise snapshots, met à jour UI
- Rendu : liste participants (avatar circulaire, nom, anneau speaking pulsant)
- Gestion cycle de vie : socket disconnect → hide overlay, reconnect → show
- Position configurable : coin (TL/TR/BL/BR), centre, coordonnées custom
- Thème : suit GTK theme (Adwaita) + CSS custom pour speaking pulse

**Architecture Interne** :
```
overlay/src/
├── main.rs                 # Entry point, GTK application
├── layer_shell.rs          # Layer-shell surface setup (click-through)
├── socket_server.rs        # Unix socket listener (async, tokio)
├── protocol.rs             # Deserialization (versioned, matches plugin)
├── ui/
│   ├── participant_list.rs # Widget liste participants
│   ├── avatar.rs           # Avatar widget (async image load)
│   └── speaking_indicator.rs # Pulse animation
├── config.rs               # Config file (~/.config/vesktop-voice-overlay/config.toml)
└── lifecycle.rs            # Vesktop lifecycle detection (socket state)
```

**Configuration** (`~/.config/vesktop-voice-overlay/config.toml`) :
```toml
[overlay]
position = "top-right"      # top-left, top-right, bottom-left, bottom-right, center, custom
custom_x = 0
custom_y = 0
max_participants = 10
avatar_size = 40

[appearance]
theme = "auto"              # auto, light, dark
speaking_pulse_ms = 1000
show_names = true

[socket]
path = "/run/user/1000/vesktop-voice-overlay.sock"  # ou $XDG_RUNTIME_DIR
```

### 2.3 Communication Plugin ↔ Overlay

| Aspect | Détail |
|--------|--------|
| **Transport** | Unix Domain Socket (AF_UNIX, SOCK_STREAM) |
| **Sécurité** | `SO_PEERCRED` validation (même UID), `SCM_RIGHTS` pour fd passing |
| **Emplacement** | `$XDG_RUNTIME_DIR/vesktop-voice-overlay.sock` |
| **Protocole** | JSON Lines (un snapshot par ligne) + version header |
| **Versioning** | Header `VESKTOP_VOICE_OVERLAY/1.0\n` puis JSON lines |
| **Reconnexion** | Plugin : backoff exponentiel (1s, 2s, 4s, 8s, 16s, max 30s) |
| **Cycle de vie** | Overlay crée socket → Plugin connecte → Snapshots flux continu |

## 3. Distribution Architecture

### Monorepo Source (GitHub)
```
vesktop-voice-overlay/
├── plugin/                    # Vencord plugin (npm package)
│   ├── package.json
│   ├── tsconfig.json
│   ├── src/
│   └── manifest.json          # Vencord manifest
├── overlay/                   # Rust crate (cargo package)
│   ├── Cargo.toml
│   ├── src/
│   └── build.rs               # Embed version from git tag
├── packaging/
│   └── aur/
│       ├── PKGBUILD           # Build overlay from source
│       ├── vesktop-voice-overlay.install
│       └── .SRCINFO
├── .github/workflows/
│   ├── ci.yml                 # Tests, lint, build
│   └── release.yml            # Tag push → build + release assets
├── README.md
├── LICENSE (GPL-3.0)
└── AGENTS.md
```

### Distribution Séparée (Utilisateur Final)

```
┌─────────────────────────────────────────────────────────────────┐
│                    MONOREPO (v1.0.0 tag)                        │
│  ├── plugin/ → vesktop-voice-overlay-plugin-1.0.0.tgz         │
│  └── overlay/ → vesktop-voice-overlay (binary)                 │
└─────────────────────────────────────────────────────────────────┘
           │                                    │
           ▼                                    ▼
┌─────────────────────────┐          ┌─────────────────────────┐
│   Vencord Plugin Store  │          │         AUR             │
│   (npm registry-like)   │          │  (archlinux.org/packages)│
│                         │          │                         │
│ Install: Vesktop UI     │          │ Install: yay -S pkg     │
│ Update: Auto (Vesktop)  │          │ Update: yay -Syu        │
└─────────────────────────┘          └─────────────────────────┘
           │                                    │
           └──────────────┬─────────────────────┘
                          ▼
              ┌───────────────────────┐
              │    UTILISATEUR FINAL  │
              │  yay -S vesktop-voice-overlay    │
              │  Vesktop → Plugins → Install     │
              └───────────────────────┘
```

### CI/CD Pipeline (`.github/workflows/release.yml`)

```yaml
on:
  push:
    tags: ['v*']

jobs:
  build-overlay:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      - name: Build overlay
        run: cargo build --release --manifest-path overlay/Cargo.toml
      - name: Upload binary
        uses: actions/upload-artifact@v4
        with:
          name: overlay-binary
          path: overlay/target/release/vesktop-voice-overlay

  build-plugin:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - run: cd plugin && npm ci && npm pack
      - name: Upload plugin
        uses: actions/upload-artifact@v4
        with:
          name: plugin-package
          path: plugin/*.tgz

  create-release:
    needs: [build-overlay, build-plugin]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            vesktop-voice-overlay
            vesktop-voice-overlay-plugin-*.tgz
          generate_release_notes: true

  publish-aur:
    needs: create-release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Update AUR PKGBUILD
        # Update pkgver, sha256sums from release assets
        # Push to AUR git repo (aur.archlinux.org/vesktop-voice-overlay.git)
```

## 4. Sécurité & Sandbox

| Mesure | Implémentation |
|--------|----------------|
| **Pas de token Discord** | Plugin lit seulement voice state interne (déjà authentifié par Vesktop) |
| **Socket user-only** | `$XDG_RUNTIME_DIR` (mode 0700), `SO_PEERCRED` validation UID |
| **Pas de réseau** | Unix socket local uniquement, pas de TCP/UDP |
| **Validation protocole** | Version header + JSON schema validation côté overlay |
| **Least privilege** | Overlay : pas de capabilities, layer-shell click-through seulement |
| **Code audit** | Plugin TypeScript (pas de `eval`, pas de `Function` constructor) |

## 5. Tests & Validation

| Niveau | Outils | Couverture |
|--------|--------|------------|
| **Unit (Plugin)** | Vitest + @types/vencord | Protocol serialization, socket logic, voice state mapping |
| **Unit (Overlay)** | cargo test + gtk4-test | Socket server, deserialization, UI widgets |
| **Integration** | Script E2E (Vesktop headless + overlay) | Full flow: join voice → overlay appears → speak → indicator |
| **Compatibilité** | Matrix: Hyprland, sway, niri, labwc | Layer-shell behavior, click-through, fullscreen games |

## 6. Évolutions Futures (Post-v1)

| Feature | Complexité | Dépendances |
|---------|------------|-------------|
| Config UI (GTK) dans overlay | Moyenne | libadwaita, settings dialog |
| Thèmes personnalisables | Faible | CSS variables |
| Historique participants | Faible | SQLite local |
| Intégration Noctalia panel | Moyenne | Plugin Noctalia séparé |
| Flatpak extension | Élevée | Flatpak Vesktop + portal |
| Support multi-canaux | Moyenne | Protocol v2 (array de canaux) |

---

*Architecture validée pour distribution Monorepo → AUR + Vencord Store. Protocole socket v1 figé à la release v1.0.0.*