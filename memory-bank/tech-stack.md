# Tech Stack — Vesktop Voice Overlay

## 1. Runtime & Langages

| Composant | Technologie | Version | Justification |
|-----------|-------------|---------|---------------|
| **Vencord Plugin** | TypeScript | 5.x | Typage strict, écosystème Vencord/Vesktop, compilation ES2022 |
| **Overlay** | Rust | 1.75+ | Performance, safety, GTK4 bindings matures (gtk4-rs), pas de GC |
| **Build Plugin** | Node.js + npm | 20 LTS | Standard Vencord, `npm pack` pour distribution |
| **Build Overlay** | Cargo | 1.75+ | Standard Rust, release optimisé, cross-compilation possible |

## 2. Frameworks & Bibliothèques

### Plugin Vencord (`plugin/`)
| Lib | Version | Usage |
|-----|---------|-------|
| `@vencord/api` | Latest | Types Vencord (voice state, settings, patches) |
| `vitest` | 1.x | Tests unitaires (protocol, socket, voice state) |
| `typescript` | 5.x | Compilation, types stricts |
| `esbuild` / `tsup` | Latest | Bundle plugin pour distribution |

### Overlay Rust (`overlay/`)
| Crate | Version | Usage |
|-------|---------|-------|
| `gtk4` | 0.7+ | UI toolkit Wayland natif (layer-shell support) |
| `gio` / `glib` | 0.19+ | Event loop, async, Unix socket |
| `tokio` | 1.x | Async runtime (socket server, reconnection logic) |
| `serde` + `serde_json` | 1.x | Sérialisation protocole (versioned JSON) |
| `image` | 0.24+ | Chargement avatars (WebP, PNG, JPEG) |
| `css-in-rust` / `gtk4-layer-shell` | Latest | CSS-in-Rust pour thèmes, layer-shell bindings |
| `anyhow` / `thiserror` | 1.x | Gestion erreurs ergonomique |

### Système (Arch Linux)
```bash
# Dépendances build
sudo pacman -S base-devel git nodejs npm rustup
rustup default stable

# Dépendances runtime overlay
sudo pacman -S gtk4 libadwaita gdk-pixbuf2

# Vencord/Vesktop (utilisateur final)
# Vesktop via AUR: yay -S vesktop
# Vencord s'installe via Vesktop UI
```

## 3. Architecture Technique

### Protocole Socket (v1)
```
Header: "VESKTOP_VOICE_OVERLAY/1.0\n"
Payload: JSON Lines (un snapshot par ligne)

Snapshot Schema:
{
  "version": 1,
  "timestamp": 1692000000000,
  "self": {
    "userId": "123456789012345678",
    "username": "Roddy",
    "avatarUrl": "https://cdn.discordapp.com/avatars/...",
    "mute": false,
    "deaf": false,
    "speaking": true
  },
  "participants": [
    {
      "userId": "987654321098765432",
      "username": "Friend",
      "avatarUrl": "https://cdn.discordapp.com/avatars/...",
      "speaking": false,
      "volume": 80
    }
  ]
}
```

### Versioning Protocole
- **Header fixe** : `VESKTOP_VOICE_OVERLAY/<major>.<minor>\n`
- **Compatibilité** : Overlay accepte `<= current_version` (forward compatible)
- **Breaking change** = major bump → nouveau tag `v2.0.0` = plugin + overlay

## 4. Build & Distribution Pipeline

### Monorepo Structure
```
vesktop-voice-overlay/
├── plugin/                    # npm workspace
│   ├── package.json           # name: "vesktop-voice-overlay-plugin"
│   ├── tsconfig.json
│   ├── src/
│   │   ├── index.ts           # Vencord entry
│   │   ├── socket.ts          # Unix socket client
│   │   ├── protocol.ts        # Types + serialization
│   │   └── voiceState.ts      # Vencord API wrapper
│   └── manifest.json          # Vencord manifest
├── overlay/                   # Cargo workspace
│   ├── Cargo.toml             # name = "vesktop-voice-overlay"
│   ├── src/
│   │   ├── main.rs
│   │   ├── layer_shell.rs
│   │   ├── socket_server.rs
│   │   ├── protocol.rs
│   │   ├── ui/
│   │   └── config.rs
│   └── build.rs               # Embed git version
├── packaging/
│   └── aur/
│       ├── PKGBUILD
│       ├── .SRCINFO
│       └── vesktop-voice-overlay.install
└── .github/workflows/
    ├── ci.yml
    └── release.yml
```

### CI/CD (GitHub Actions)

| Workflow | Trigger | Actions |
|----------|---------|---------|
| `ci.yml` | PR, push main | `cargo test`, `cargo clippy`, `npm test`, `npm run lint`, build verification |
| `release.yml` | Tag `v*` | Build overlay (release), `npm pack` plugin, create GitHub Release avec assets, publish AUR |

### Artefacts Release
| Artefact | Origine | Destination |
|----------|---------|-------------|
| `vesktop-voice-overlay` (binary) | `cargo build --release` | GitHub Release + AUR |
| `vesktop-voice-overlay-plugin-<version>.tgz` | `npm pack` | GitHub Release + Vencord Store |

### AUR PKGBUILD (overlay only)
```bash
# packaging/aur/PKGBUILD
pkgname=vesktop-voice-overlay
pkgver=1.0.0
pkgrel=1
pkgdesc="Wayland-native voice activity overlay for Vesktop"
arch=(x86_64)
url="https://github.com/Roddygithub/vesktop-voice-overlay"
license=(GPL-3.0-only)
depends=(gtk4 libadwaita glib2)
makedepends=(cargo git)
source=("vesktop-voice-overlay-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('...')

build() {
  cd "$pkgname-$pkgver/overlay"
  cargo build --release --locked
}

package() {
  cd "$pkgname-$pkgver/overlay"
  install -Dm755 "target/release/vesktop-voice-overlay" "$pkgdir/usr/bin/vesktop-voice-overlay"
  install -Dm644 "../LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
```

## 5. Déploiement Utilisateur Final

### Installation (2 commandes)
```bash
# 1. Overlay binaire (AUR)
yay -S vesktop-voice-overlay

# 2. Plugin Vencord (via Vesktop UI)
# Vesktop → Settings → Plugins → "Install from Store" → chercher "Voice Overlay"
# OU: "Install from URL" → https://github.com/Roddygithub/vesktop-voice-overlay/releases/download/v1.0.0/vesktop-voice-overlay-plugin-1.0.0.tgz
```

### Mises à jour
| Composant | Mécanisme |
|-----------|-----------|
| Overlay | `yay -Syu` (pacman/AUR) |
| Plugin | Auto via Vesktop (vérification quotidienne) ou manuel dans Settings → Plugins |

## 6. Qualité & Tests

| Niveau | Outils | Commandes |
|--------|--------|-----------|
| **Lint Plugin** | ESLint + @typescript-eslint | `npm run lint` |
| **Test Plugin** | Vitest | `npm test` |
| **Vencord Integration** | pnpm build (pinned rev) | Clone Vencord → copy plugin → `pnpm build` |
| **Lint Overlay** | Clippy + rustfmt | `cargo clippy -- -D warnings && cargo fmt --check` |
| **Test Overlay** | cargo test | `cargo test` |
| **Build Check** | cargo build --release | `cargo build --release --locked` |

## 6. Versions & Compatibilité

| Composant | Version Min | Testé | Notes |
|-----------|-------------|-------|-------|
| Rust | 1.75+ | 1.79 | MSRV = 1.75 |
| Node.js | 20 LTS | 20.15 | LTS actif |
| GTK4 | 4.12+ | 4.14 | layer-shell stable |
| Vencord | 1.15.2+ | Pinned `ef29bbeb` | Userplugin in `src/userplugins/` |
| Vesktop | Latest | Stable | Electron 28+ |
| Wayland Compositor | wlroots 0.17+ | Hyprland, sway, niri | layer-shell v1 |

## 7. Alternatives Rejetées

| Alternative | Raison Rejet |
|-------------|--------------|
| **Electron overlay** | Lourd, pas natif Wayland, pas click-through fiable |
| **Python + GTK** | Performance, déploiement binaire complexe, pas de safety |
| **C++ + GTK** | Complexité build, safety mémoire, temps dev |
| **Single package AUR (plugin + overlay)** | Vencord plugin doit être dans `~/.config/Vencord/plugins/`, pas géré par pacman |
| **Flatpak only** | Vesktop souvent en AUR/binary, pas Flatpak ; complexité portal |
| **WebSocket / TCP** | Sécurité (exposition réseau), complexité, pas besoin |

---

*Stack validée pour Monorepo TypeScript/Rust → AUR (overlay) + Vencord Store (plugin). Protocole socket v1 figé à v1.0.0.*