# Architecture — Vesktop Voice Overlay

## 1. Vue d'Ensemble Système

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           VESKTOP (Electron + Vencord)                      │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Renderer Process (WebView)                                         │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │  Vencord Plugin (TypeScript)                                │   │   │
│  │  │  - Accède aux stores voice internes (VoiceStateStore/UserStore) │   │
│  │  │  - Sérialise snapshots versionnés (JSON)                   │   │   │
│  │  │  - Envoie via Unix socket (credentials passing)            │   │   │
│  │  │  - Gère reconnexion auto                                    │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                     Unix Socket (SO_PEERCRED, user-only)
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

## Runtime Threading Constraint

The socket accept loop uses blocking Unix I/O and must run outside GTK's main
context. It dispatches validated commands through the channel consumed by the
GTK event loop; `ClientConnected`, `UpdateSnapshot`, and lifecycle visibility
changes must therefore remain GTK-thread operations.

Avatar network I/O is an exception to GTK-thread work: HTTP fetches run on a
Tokio-backed worker, while decoded textures and widget updates return to the
GTK context.

The Voice Widget uses a transparent layer-shell window and scrolled viewport;
each visible participant owns its compact translucent row. This prevents GTK
theme backgrounds from producing a second opaque card around the widget.

Discord avatars are decoded as RGBA and passed directly to GdkPixbuf. Avoiding
the Cairo ARGB32 intermediate prevents channel-order swaps on little-endian
systems.

Vencord persists Voice Widget preferences and sends a separate `type:
"settings"` message through the same-user Unix socket. The overlay applies
only display modes and layer-shell position values from this message; versioned
voice snapshots remain authoritative participant data.

## 2. Composants Détaillés

### 2.1 Vencord Plugin (`plugin/`)

**Responsabilités** :
- Accès aux stores voice internes Vesktop (`VoiceStateStore`, `UserStore`)
- Détection changements : join/leave/mute/deaf/speaking
- Sérialisation snapshots versionnés (JSON) avec timestamp + version protocole
- Envoi via Unix socket user-only, validé par `SO_PEERCRED`
- Gestion reconnexion : exponential backoff, max 5 tentatives, puis pause
- Cycle de vie : se connecte au démarrage Vesktop, nettoie socket à l'arrêt

**Architecture Interne** :
```
plugin/src/
├── index.ts              # Entry point Vencord (definePlugin, flux events)
├── native.ts             # Electron IPC native module (IpcMainInvokeEvent)
├── protocol.ts           # Types TypeScript + serialization (v1)
├── voiceState.ts         # Vencord stores (VoiceStateStore, UserStore) + speaking set
└── snapshot.ts           # Création snapshots (participants, speaking, self)
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
avatar_size = 28
user_display = "speaking_only"
name_display = "speaking_only"
avatar_size_mode = "small"

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
├── plugin/                    # Vencord userplugin (source, not npm package)
│   ├── package.json           # Dev deps (vitest, eslint) only
│   ├── tsconfig.json          # Local IDE, not authoritative
│   └── src/
│       ├── index.ts           # definePlugin entry, flux events
│       ├── native.ts          # Electron IPC native module
│       ├── protocol.ts        # Shared protocol types
│       ├── voiceState.ts      # Vencord stores adapter
│       └── snapshot.ts        # Snapshot builder
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
│   ├── ci.yml                 # Tests, lint, Vencord-integrated build
│   └── release.yml            # Tag push → build + release assets
├── README.md
├── LICENSE (GPL-3.0)
└── AGENTS.md
```

> **Plugin is a Vencord source userplugin**, not a standalone npm package.
> For CI, plugin files are copied into Vencord's `src/userplugins/vesktopVoiceOverlay/`
> and built via `pnpm build` within the Vencord source tree.
> Symlinks do not work because esbuild resolves `@utils/*`/`@webpack` relative to the
> real file path, not the symlink path.
> Vencord compatibility baseline: pinned revision `ef29bbeb` (v1.15.2).

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

### CI/CD Pipeline (`.github/workflows/ci.yml`)

```yaml
# Vencord compatibility baseline
VENCORD_REV: 'ef29bbeb'

jobs:
  lint-and-test:
    # Rust fmt, clippy, build, test + Plugin lint, vitest
    # Standalone typecheck removed (Vencord modules not resolvable)

  vencord-integration:
    needs: lint-and-test
    # Clones pinned Vencord, copies plugin, runs pnpm build
    # Fails if plugin is incompatible with pinned Vencord revision

  build-release:
    # Only on push to main: cargo build --release
```

> **Standalone typecheck is not authoritative.** The plugin imports Vencord-internal
> modules (`@utils/*`, `@webpack`) that only resolve within Vencord's esbuild build.
> The `vencord-integration` job is the external compatibility gate.
> To update the Vencord baseline, change `VENCORD_REV` in `ci.yml` and validate locally.

### Dependency maintenance policy (`.github/dependabot.yml`, 2026-08-25)

```yaml
# Monthly cadence, one grouped PR per ecosystem for minor+patch.
npm (/plugin):        minor+patch grouped; semver-major ignored
cargo (/overlay):     minor+patch grouped; majors individual, limit 3
github-actions (/.):  minor+patch grouped; semver-major ignored
```

Rationale:
- **Majors are suppressed** on npm and Actions until adopted deliberately
  (eslint ≥9 needs flat-config migration; release-path actions can only be
  validated during a release window). Security updates are never grouped or
  ignored — they always arrive individually.
- **Cargo majors stay individual** but capped: gtk-rs core crates (glib, gio,
  gtk4, gdk4, libadwaita, gdk-pixbuf) must be bumped **in lockstep** by hand;
  solo Dependabot bumps of any gtk-rs crate duplicate the glib stack in
  Cargo.lock and must be rejected.
- SHA-pinned actions get digest PRs individually (grouping does not apply to
  them); codeql-action pins must always move init+analyze together.

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
| **Unit (Plugin)** | Vitest | Protocol serialization |
| **Unit (Overlay)** | cargo test | Socket server, deserialization, lifecycle |
| **Vencord Integration** | `pnpm build` in pinned Vencord source | Plugin discovery, native IPC, flux events, full build |
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
