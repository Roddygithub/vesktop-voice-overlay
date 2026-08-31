#!/usr/bin/env bash
set -Eeuo pipefail
IFS=$'\n\t'

PRODUCT_NAME="Discord Voice Overlay"
INSTALLER_VERSION="1.3.0"
OVERLAY_VERSION="${DVO_OVERLAY_VERSION:-1.3.0}"
VENCORD_REPO="${DVO_VENCORD_REPO:-https://github.com/Vendicated/Vencord.git}"
VENCORD_REV="${DVO_VENCORD_REV:-ef29bbeb6119cfb53d1273ed78147bcc97d91261}"
RELEASE_REPO="Roddygithub/vesktop-voice-overlay"
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)

: "${HOME:?HOME must be set}"

CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
STATE_HOME="${XDG_STATE_HOME:-$HOME/.local/state}"
SYSTEM_ROOT="${DVO_SYSTEM_ROOT:-}"
MANAGED_ROOT="${DVO_MANAGED_ROOT:-$DATA_HOME/discord-voice-overlay}"
STATE_DIR="${STATE_HOME}/discord-voice-overlay"
STATE_FILE="${STATE_DIR}/state.env"
SERVICE_PATH="${CONFIG_HOME}/systemd/user/vesktop-voice-overlay.service"
BACKUP_ROOT="${MANAGED_ROOT}/backups"
VENCORD_ROOT="${MANAGED_ROOT}/vencord"
VENCORD_DIST="${VENCORD_ROOT}/dist"
CURRENT_OVERLAY="${MANAGED_ROOT}/current/vesktop-voice-overlay"
VENCORD_SETTINGS_PATH="${CONFIG_HOME}/Vencord/settings/settings.json"
VENCORD_SETTINGS_BACKUP="${BACKUP_ROOT}/vencord-settings.json"

DRY_RUN=0
ASSUME_YES=0
REQUESTED_CLIENT=""
SELECTED_CLIENT=""
OVERLAY_EXECUTABLE=""
OVERLAY_METHOD=""
DISCORD_TARGET=""
DISCORD_ORIGINAL_HASH=""
VESKTOP_STATE_PATH="${CONFIG_HOME}/vesktop/state.json"
VESKTOP_STATE_HAD_FILE=0
TEMP_PATHS=()
VENCORD_SETTINGS_CHANGED=0
VENCORD_SETTINGS_HAD_FILE=0
VENCORD_SETTINGS_ORIGINAL_HASH=""
VENCORD_SETTINGS_MANAGED_HASH=""
VENCORD_SETTINGS_ROLLBACK_PATH=""
VENCORD_SETTINGS_MODIFIED_THIS_RUN=0
INSTALL_COMPLETED=0

CLIENT_VESKTOP_NATIVE=0
CLIENT_DISCORD_NATIVE=0
CLIENT_VESKTOP_FLATPAK=0
CLIENT_DISCORD_FLATPAK=0

info() { printf '%s\n' "$*"; }
warn() { printf 'Warning: %s\n' "$*" >&2; }
die() { printf 'Error: %s\n' "$*" >&2; exit 1; }

[[ "$OVERLAY_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "invalid overlay version: $OVERLAY_VERSION"
[[ "$VENCORD_REV" =~ ^[[:xdigit:]]{40}$ ]] || die "invalid Vencord revision: $VENCORD_REV"

usage() {
    cat <<'EOF'
Discord Voice Overlay installer/manager

Usage:
  ./install.sh [install|update|repair|status|doctor|uninstall]

Options:
  --client vesktop|discord  Select the client to manage
  --yes                     Confirm client configuration and replacement
  --dry-run                 Show changes without modifying the system
  --version                Show installer version
  --help                   Show this help

With no command, install is used. AUR and package-manager hooks are not
required; the manager installs only user-owned components.
EOF
}

version() { printf '%s installer %s\n' "$PRODUCT_NAME" "$INSTALLER_VERSION"; }

cleanup() {
    local path
    if (( VENCORD_SETTINGS_MODIFIED_THIS_RUN && !INSTALL_COMPLETED )); then
        if [[ -n "$VENCORD_SETTINGS_ROLLBACK_PATH" && -f "$VENCORD_SETTINGS_ROLLBACK_PATH" ]]; then
            mkdir -p -- "$(dirname -- "$VENCORD_SETTINGS_PATH")"
            cp -p -- "$VENCORD_SETTINGS_ROLLBACK_PATH" "$VENCORD_SETTINGS_PATH"
        elif (( ! VENCORD_SETTINGS_HAD_FILE )); then
            rm -f -- "$VENCORD_SETTINGS_PATH"
        fi
    fi
    for path in "${TEMP_PATHS[@]}"; do
        if [[ "$path" == "$MANAGED_ROOT/.staging."* ||
            "$path" == "${TMPDIR:-/tmp}/discord-voice-overlay."* ]]; then
            rm -rf -- "$path"
        fi
    done
}
trap cleanup EXIT

canonical() { realpath -m -- "$1"; }

require_absolute() {
    [[ "$1" == /* ]] || die "path must be absolute: $1"
}

safe_managed_path() {
    local path root
    path=$(canonical "$1")
    root=$(canonical "$MANAGED_ROOT")
    [[ "$path" == "$root" || "$path" == "$root/"* ]] || return 1
    [[ "$path" != "/" && "$path" != "$HOME" && "$path" != "$DATA_HOME" ]] || return 1
}

remove_owned_tree() {
    local path
    [[ ! -L "$1" ]] || die "refusing to remove symlink path: $1"
    path=$(canonical "$1")
    safe_managed_path "$path" || die "refusing to remove unowned path: $path"
    [[ "$path" == "$(canonical "$MANAGED_ROOT")" ||
        "$path" == "$(canonical "$MANAGED_ROOT")/"* ]] ||
        die "refusing to remove path outside managed tree: $path"
    [[ -e "$path" || -L "$path" ]] || return 0
    (( DRY_RUN )) && { info "[dry-run] remove $path"; return 0; }
    rm -rf -- "$1"
}

need_cmd() { command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"; }

require_user() {
    local managed_root data_root
    [[ "$(id -u)" -ne 0 ]] || die "run this manager as your normal user, not as root"
    require_absolute "$CONFIG_HOME"
    require_absolute "$DATA_HOME"
    require_absolute "$STATE_HOME"
    require_absolute "$MANAGED_ROOT"
    [[ "$MANAGED_ROOT" == "$DATA_HOME/"* ]] ||
        die "managed data must remain under XDG_DATA_HOME: $MANAGED_ROOT"
    [[ ! -L "$MANAGED_ROOT" ]] || die "managed data path must not be a symlink: $MANAGED_ROOT"
    managed_root=$(canonical "$MANAGED_ROOT")
    data_root=$(canonical "$DATA_HOME")
    [[ "$managed_root" == "$data_root/"* ]] ||
        die "managed data must resolve under XDG_DATA_HOME: $MANAGED_ROOT"
}

system_path() {
    if [[ -n "$SYSTEM_ROOT" ]]; then
        printf '%s%s\n' "$SYSTEM_ROOT" "$1"
    else
        printf '%s\n' "$1"
    fi
}

state_get() {
    local wanted="$1" key value
    [[ -f "$STATE_FILE" ]] || return 1
    while IFS='=' read -r key value; do
        [[ "$key" == "$wanted" ]] || continue
        printf '%s\n' "$value"
        return 0
    done < "$STATE_FILE"
    return 1
}

state_is_valid() {
    local client overlay service vencord settings_path settings_changed settings_had_file
    [[ -f "$STATE_FILE" ]] || return 1
    [[ "$(state_get schema 2>/dev/null || true)" == "1" ]] || return 1
    [[ "$(state_get managed_root 2>/dev/null || true)" == "$MANAGED_ROOT" ]] || return 1
    client=$(state_get client 2>/dev/null || true)
    [[ "$client" == vesktop || "$client" == discord ]] || return 1
    overlay=$(state_get overlay_path 2>/dev/null || true)
    if [[ "$overlay" == "$MANAGED_ROOT/"* ]]; then
        safe_managed_path "$overlay" || return 1
    elif [[ "$overlay" != /usr/bin/vesktop-voice-overlay &&
        "$overlay" != "$HOME/.local/bin/vesktop-voice-overlay" ]]; then
        return 1
    fi
    service=$(state_get service_path 2>/dev/null || true)
    [[ "$service" == "$SERVICE_PATH" ]] || return 1
    vencord=$(state_get vencord_root 2>/dev/null || true)
    [[ "$vencord" == "$VENCORD_ROOT" ]] || return 1
    settings_path=$(state_get vencord_settings_path 2>/dev/null || true)
    [[ "$settings_path" == "$VENCORD_SETTINGS_PATH" ]] || return 1
    settings_changed=$(state_get vencord_settings_changed 2>/dev/null || true)
    settings_had_file=$(state_get vencord_settings_had_file 2>/dev/null || true)
    [[ "$settings_changed" == 0 || "$settings_changed" == 1 ]] || return 1
    [[ "$settings_had_file" == 0 || "$settings_had_file" == 1 ]] || return 1
    if [[ "$settings_changed" == 1 ]]; then
        [[ "$(state_get vencord_settings_managed_hash 2>/dev/null || true)" =~ ^[[:xdigit:]]{64}$ ]] || return 1
        if [[ "$settings_had_file" == 1 ]]; then
            [[ "$(state_get vencord_settings_original_hash 2>/dev/null || true)" =~ ^[[:xdigit:]]{64}$ ]] || return 1
            [[ -f "$VENCORD_SETTINGS_BACKUP" ]] || return 1
        fi
    fi
    if [[ "$client" == discord ]]; then
        discord_target_is_safe "$(state_get discord_target 2>/dev/null || true)" || return 1
        [[ "$(state_get discord_original_hash 2>/dev/null || true)" =~ ^[[:xdigit:]]{64}$ ]] || return 1
    fi
}

ensure_state_usable() {
    [[ ! -L "$STATE_FILE" ]] || die "installer state path is a symlink: $STATE_FILE"
    [[ ! -e "$STATE_FILE" || -f "$STATE_FILE" ]] || die "installer state is not a regular file: $STATE_FILE"
    [[ ! -f "$STATE_FILE" ]] || state_is_valid ||
        die "installer state is missing or corrupt; refusing to guess ownership: $STATE_FILE"
}

write_state() {
    local tmp
    (( DRY_RUN )) && { info "[dry-run] write installer state: $STATE_FILE"; return 0; }
    mkdir -p -- "$STATE_DIR"
    chmod 700 "$STATE_DIR"
    tmp=$(mktemp "$STATE_DIR/.state.XXXXXX")
    printf 'schema=1\ninstaller_version=%s\noverlay_version=%s\noverlay_path=%s\noverlay_method=%s\nclient=%s\nvencord_root=%s\nvencord_dist=%s\nvencord_revision=%s\nservice_path=%s\nvesktop_state_path=%s\nvesktop_state_had_file=%s\ndiscord_target=%s\ndiscord_original_hash=%s\nintegration_method=%s\nmanaged_root=%s\nvencord_settings_path=%s\nvencord_settings_changed=%s\nvencord_settings_had_file=%s\nvencord_settings_original_hash=%s\nvencord_settings_managed_hash=%s\n' \
        "$INSTALLER_VERSION" "$OVERLAY_VERSION" "$OVERLAY_EXECUTABLE" "$OVERLAY_METHOD" \
        "$SELECTED_CLIENT" "$VENCORD_ROOT" "$VENCORD_DIST" "$VENCORD_REV" \
        "$SERVICE_PATH" "$VESKTOP_STATE_PATH" "$VESKTOP_STATE_HAD_FILE" \
        "$DISCORD_TARGET" "$DISCORD_ORIGINAL_HASH" \
        "$( [[ "$SELECTED_CLIENT" == "discord" ]] && printf 'vencord-official-inject' || printf 'vesktop-state' )" \
        "$MANAGED_ROOT" "$VENCORD_SETTINGS_PATH" "$VENCORD_SETTINGS_CHANGED" \
        "$VENCORD_SETTINGS_HAD_FILE" "$VENCORD_SETTINGS_ORIGINAL_HASH" \
        "$VENCORD_SETTINGS_MANAGED_HASH" > "$tmp"
    chmod 600 "$tmp"
    mv -f -- "$tmp" "$STATE_FILE"
}

confirm() {
    local prompt="$1" answer
    (( ASSUME_YES )) && return 0
    (( DRY_RUN )) && { info "[dry-run] $prompt"; return 0; }
    [[ -t 0 && -t 1 ]] || die "$prompt Re-run with --yes after reviewing the target."
    read -r -p "$prompt [y/N] " answer
    [[ "$answer" == "y" || "$answer" == "Y" ]]
}

detect_flatpaks() {
    local apps
    command -v flatpak >/dev/null 2>&1 || return 0
    apps=$(flatpak list --app --columns=application 2>/dev/null || true)
    grep -Fxq 'dev.vencord.Vesktop' <<<"$apps" && CLIENT_VESKTOP_FLATPAK=1
    grep -Fxq 'com.discordapp.Discord' <<<"$apps" && CLIENT_DISCORD_FLATPAK=1
}

detect_native_clients() {
    local vesktop_bin discord_bin
    vesktop_bin=$(system_path /usr/bin/vesktop)
    discord_bin=$(system_path /usr/bin/discord)
    if [[ -x "$vesktop_bin" && -d "$(system_path /usr/lib/vesktop)" ]] &&
        package_owns "$vesktop_bin" vesktop; then
        CLIENT_VESKTOP_NATIVE=1
    fi
    if [[ -x "$discord_bin" && -d "$(system_path /usr/share/discord)" ]] &&
        package_owns "$discord_bin" discord; then
        CLIENT_DISCORD_NATIVE=1
    fi
    detect_flatpaks
}

client_label() { [[ "$1" == "discord" ]] && printf 'Discord Desktop' || printf 'Vesktop'; }

package_owns() {
    local path="$1" package="$2" owner
    [[ -n "$SYSTEM_ROOT" ]] && return 0
    command -v pacman >/dev/null 2>&1 || return 0
    owner=$(pacman -Qo -- "$path" 2>/dev/null || true)
    [[ "$owner" == *" is owned by $package "* ]]
}

print_clients() {
    if (( CLIENT_DISCORD_NATIVE )); then info '  [native] Discord Desktop'; fi
    if (( CLIENT_VESKTOP_NATIVE )); then info '  [native] Vesktop'; fi
    if (( CLIENT_DISCORD_FLATPAK )); then info '  [unsupported] Discord Flatpak'; fi
    if (( CLIENT_VESKTOP_FLATPAK )); then info '  [unsupported] Vesktop Flatpak'; fi
    if (( ! CLIENT_DISCORD_NATIVE && ! CLIENT_VESKTOP_NATIVE &&
        ! CLIENT_DISCORD_FLATPAK && ! CLIENT_VESKTOP_FLATPAK )); then
        info '  none detected'
    fi
}

client_available() {
    [[ "$1" == "discord" && "$CLIENT_DISCORD_NATIVE" -eq 1 ]] ||
        [[ "$1" == "vesktop" && "$CLIENT_VESKTOP_NATIVE" -eq 1 ]]
}

select_client() {
    local choices=()
    client_available discord && choices+=(discord)
    client_available vesktop && choices+=(vesktop)

    if [[ -n "$REQUESTED_CLIENT" ]]; then
        client_available "$REQUESTED_CLIENT" || {
            if [[ "$REQUESTED_CLIENT" == "discord" && "$CLIENT_DISCORD_FLATPAK" -eq 1 ]]; then
                die 'Discord Flatpak is detected but unsupported; install the native Arch package'
            fi
            if [[ "$REQUESTED_CLIENT" == "vesktop" && "$CLIENT_VESKTOP_FLATPAK" -eq 1 ]]; then
                die 'Vesktop Flatpak is detected but unsupported; install the native Arch package'
            fi
            die "native $(client_label "$REQUESTED_CLIENT") installation not detected"
        }
        SELECTED_CLIENT="$REQUESTED_CLIENT"
        return 0
    fi

    case "${#choices[@]}" in
        0) die 'no supported native client detected; Flatpak, AppImage, and custom installations are unsupported' ;;
        1) SELECTED_CLIENT="${choices[0]}" ;;
        *)
            [[ -t 0 && -t 1 ]] || die 'both native clients are installed; select one with --client vesktop or --client discord'
            info 'Both supported clients are detected.'
            info '1) Discord Desktop'
            info '2) Vesktop'
            local choice
            read -r -p 'Choose client [1-2]: ' choice
            case "$choice" in
                1) SELECTED_CLIENT=discord ;;
                2) SELECTED_CLIENT=vesktop ;;
                *) die 'invalid client selection' ;;
            esac
            ;;
    esac
}

latest_discord_target() {
    local resources dir
    mapfile -t resources < <(printf '%s\n' "$CONFIG_HOME"/discord/app-*/resources | sort -V -r)
    for resources in "${resources[@]}"; do
        [[ -d "$resources" && -f "$resources/app.asar" ]] || continue
        dir=$(canonical "${resources%/resources}")
        [[ "$dir" == "$(canonical "$CONFIG_HOME/discord")/app-"* ]] || continue
        printf '%s\n' "$resources"
        return 0
    done
    return 1
}

discord_target_is_safe() {
    local target root
    target=$(canonical "$1")
    root=$(canonical "$CONFIG_HOME/discord")
    [[ ! -L "$1" && ! -L "$1/app.asar" && ! -L "$1/_app.asar" &&
        "$target" == "$root"/app-*/resources && -d "$target" ]]
}

sha256_file() { sha256sum -- "$1" | awk '{print $1}'; }

verify_checksum() {
    local file="$1" sums="$2" asset="$3" expected='' actual
    local hash name
    while IFS=' ' read -r hash name; do
        name="${name#\*}"
        [[ "$name" == "$asset" ]] || continue
        expected="$hash"
        break
    done < "$sums"
    [[ "$expected" =~ ^[[:xdigit:]]{64}$ ]] || die "checksum entry missing for $asset"
    actual=$(sha256_file "$file")
    [[ "$actual" == "$expected" ]] || die "checksum verification failed for $asset"
}

existing_overlay() {
    local candidate output
    for candidate in "$(system_path /usr/bin/vesktop-voice-overlay)" "$HOME/.local/bin/vesktop-voice-overlay"; do
        [[ -x "$candidate" ]] || continue
        output=$("$candidate" --version 2>/dev/null || true)
        [[ "$output" == *"$OVERLAY_VERSION"* ]] || continue
        printf '%s\n' "$candidate"
        return 0
    done
    return 1
}

install_managed_overlay() {
    local source="$1" version_dir tmp
    version_dir="$MANAGED_ROOT/versions/$OVERLAY_VERSION"
    [[ ! -L "$MANAGED_ROOT/versions" && ! -L "$version_dir" &&
        ! -L "$MANAGED_ROOT/current" ]] || die 'managed overlay paths must not be symlinks'
    if (( DRY_RUN )); then
        info "[dry-run] install overlay under $version_dir"
        OVERLAY_EXECUTABLE="$CURRENT_OVERLAY"
        OVERLAY_METHOD="managed-release"
        return 0
    fi
    mkdir -p -- "$version_dir" "$MANAGED_ROOT/current"
    tmp=$(mktemp "$version_dir/.overlay.XXXXXX")
    cp -- "$source" "$tmp"
    chmod 755 "$tmp"
    mv -f -- "$tmp" "$version_dir/vesktop-voice-overlay"
    ln -sfn -- "$version_dir/vesktop-voice-overlay" "$CURRENT_OVERLAY"
    OVERLAY_EXECUTABLE="$CURRENT_OVERLAY"
    OVERLAY_METHOD="managed-release"
}

download_overlay() {
    local tmpdir sums binary base
    need_cmd curl
    need_cmd sha256sum
    if (( DRY_RUN )); then
        info "[dry-run] download and verify overlay v$OVERLAY_VERSION from GitHub"
        install_managed_overlay /dev/null
        return 0
    fi
    tmpdir=$(mktemp -d "${TMPDIR:-/tmp}/discord-voice-overlay.XXXXXX")
    TEMP_PATHS+=("$tmpdir")
    base="https://github.com/$RELEASE_REPO/releases/download/v$OVERLAY_VERSION"
    sums="$tmpdir/SHA256SUMS"
    binary="$tmpdir/vesktop-voice-overlay"
    curl --fail --location --proto '=https' --proto-redir '=https' --tlsv1.2 --retry 2 --output "$sums" "$base/SHA256SUMS"
    curl --fail --location --proto '=https' --proto-redir '=https' --tlsv1.2 --retry 2 --output "$binary" "$base/vesktop-voice-overlay"
    verify_checksum "$binary" "$sums" vesktop-voice-overlay
    install_managed_overlay "$binary"
}

ensure_overlay() {
    local existing
    if [[ -L "$CURRENT_OVERLAY" && -x "$CURRENT_OVERLAY" &&
        "$(canonical "$CURRENT_OVERLAY")" == "$MANAGED_ROOT/versions/"* ]]; then
        if "$CURRENT_OVERLAY" --version 2>/dev/null | grep -Fq "$OVERLAY_VERSION"; then
            OVERLAY_EXECUTABLE="$CURRENT_OVERLAY"
            OVERLAY_METHOD="managed-release"
            info 'Overlay: existing managed release is current.'
            return 0
        fi
    fi
    if [[ -n "${DVO_OVERLAY_BINARY:-}" ]]; then
        [[ -x "$DVO_OVERLAY_BINARY" ]] || die "DVO_OVERLAY_BINARY is not executable"
        install_managed_overlay "$DVO_OVERLAY_BINARY"
        info 'Overlay: installed supplied local binary.'
        return 0
    fi
    if existing=$(existing_overlay); then
        OVERLAY_EXECUTABLE="$existing"
        OVERLAY_METHOD="existing-unmanaged"
        info "Overlay: using existing compatible binary $existing."
        return 0
    fi
    download_overlay
    info 'Overlay: downloaded and checksum-verified release binary.'
}

plugin_files=(
    index.ts
    native.ts
    protocol.ts
    resendCache.ts
    voiceState.ts
)

plugin_hash() {
    local file
    for file in "${plugin_files[@]}"; do
        [[ -f "$SCRIPT_DIR/plugin/src/$file" ]] || die "missing plugin source: $file"
        sha256sum -- "$SCRIPT_DIR/plugin/src/$file"
    done | sha256sum | awk '{print $1}'
}

managed_vencord_valid() {
    local marker revision hash
    marker="$VENCORD_ROOT/.discord-voice-overlay-build"
    [[ -f "$marker" && -d "$VENCORD_DIST" ]] || return 1
    revision=$(awk -F= '$1 == "revision" {print $2}' "$marker")
    hash=$(awk -F= '$1 == "plugin_hash" {print $2}' "$marker")
    [[ "$revision" == "$VENCORD_REV" && "$hash" == "$(plugin_hash)" ]] || return 1
    [[ -f "$VENCORD_DIST/vencordDesktopMain.js" &&
        -f "$VENCORD_DIST/vencordDesktopRenderer.js" ]] || return 1
    local file
    for file in "${plugin_files[@]}"; do
        cmp -s -- "$SCRIPT_DIR/plugin/src/$file" "$VENCORD_ROOT/src/userplugins/vesktopVoiceOverlay/$file" || return 1
    done
    grep -Fq 'VesktopVoiceOverlay' "$VENCORD_DIST/vencordDesktopMain.js" &&
        grep -Fq 'VesktopVoiceOverlay' "$VENCORD_DIST/vencordDesktopRenderer.js"
}

managed_vencord_safe_to_replace() {
    local entry status path file
    [[ ! -e "$VENCORD_ROOT" ]] && return 0
    [[ -f "$VENCORD_ROOT/.discord-voice-overlay-build" && -d "$VENCORD_ROOT/.git" ]] || return 1
    if [[ -d "$VENCORD_ROOT/src/userplugins" ]]; then
        for entry in "$VENCORD_ROOT/src/userplugins"/*; do
            [[ -e "$entry" ]] || continue
            [[ "$(basename -- "$entry")" == vesktopVoiceOverlay ]] || return 1
        done
    fi
    while IFS=' ' read -r _ path; do
        [[ -n "$path" ]] || continue
        case "$path" in
            .discord-voice-overlay-build|\
            src/userplugins/vesktopVoiceOverlay/index.ts|\
            src/userplugins/vesktopVoiceOverlay/native.ts|\
            src/userplugins/vesktopVoiceOverlay/protocol.ts|\
            src/userplugins/vesktopVoiceOverlay/resendCache.ts|\
            src/userplugins/vesktopVoiceOverlay/voiceState.ts) ;;
            *) return 1 ;;
        esac
    done < <(git -C "$VENCORD_ROOT" status --porcelain)
    for file in "${plugin_files[@]}"; do
        cmp -s -- "$SCRIPT_DIR/plugin/src/$file" "$VENCORD_ROOT/src/userplugins/vesktopVoiceOverlay/$file" || return 1
    done
}

build_managed_vencord() {
    local stage stage_v remote actual marker hash
    need_cmd git
    need_cmd pnpm
    need_cmd sha256sum
    if [[ "${DVO_TEST_MODE:-0}" != 1 ]]; then
        [[ "$VENCORD_REPO" == "https://github.com/Vendicated/Vencord.git" ||
            "$VENCORD_REPO" == "https://github.com/Vendicated/Vencord" ]] ||
            die 'managed Vencord repository must be the official GitHub repository'
    fi
    if (( DRY_RUN )); then
        info "[dry-run] clone Vencord at $VENCORD_REV and build the managed plugin"
        return 0
    fi
    mkdir -p -- "$MANAGED_ROOT"
    [[ ! -L "$VENCORD_ROOT" ]] || die "managed Vencord path must not be a symlink: $VENCORD_ROOT"
    if [[ -e "$VENCORD_ROOT" && ! -f "$VENCORD_ROOT/.discord-voice-overlay-build" ]]; then
        die "managed Vencord path exists without installer ownership marker: $VENCORD_ROOT"
    fi
    managed_vencord_safe_to_replace ||
        die 'managed Vencord contains unrecognized user customizations; refusing replacement'
    stage=$(mktemp -d "$MANAGED_ROOT/.staging.XXXXXX")
    TEMP_PATHS+=("$stage")
    stage_v="$stage/vencord"
    git clone "$VENCORD_REPO" "$stage_v"
    git -C "$stage_v" checkout --detach "$VENCORD_REV"
    remote=$(git -C "$stage_v" remote get-url origin)
    if [[ "${DVO_TEST_MODE:-0}" == 1 ]]; then
        [[ "$remote" == "$VENCORD_REPO" ]] || die 'test managed checkout remote mismatch'
    else
        [[ "$remote" == "https://github.com/Vendicated/Vencord.git" ||
            "$remote" == "https://github.com/Vendicated/Vencord" ]] ||
            die "managed checkout remote is not official Vencord"
    fi
    actual=$(git -C "$stage_v" rev-parse HEAD)
    [[ "$actual" == "$VENCORD_REV" ]] || die "managed Vencord checkout revision mismatch"
    mkdir -p -- "$stage_v/src/userplugins/vesktopVoiceOverlay"
    local file
    for file in "${plugin_files[@]}"; do
        cp -- "$SCRIPT_DIR/plugin/src/$file" "$stage_v/src/userplugins/vesktopVoiceOverlay/$file"
    done
    (cd "$stage_v" && VENCORD_USER_DATA_DIR="$MANAGED_ROOT/vencord-data" VENCORD_DEV_INSTALL=1 pnpm install --frozen-lockfile)
    (cd "$stage_v" && VENCORD_USER_DATA_DIR="$MANAGED_ROOT/vencord-data" VENCORD_DEV_INSTALL=1 pnpm build)
    printf '{}\n' > "$stage_v/dist/package.json"
    grep -Fq 'VesktopVoiceOverlay' "$stage_v/dist/vencordDesktopMain.js" || die 'plugin missing from Vencord main bundle'
    grep -Fq 'VesktopVoiceOverlay' "$stage_v/dist/vencordDesktopRenderer.js" || die 'plugin missing from Vencord renderer bundle'
    hash=$(plugin_hash)
    marker="$stage_v/.discord-voice-overlay-build"
    printf 'revision=%s\nplugin_hash=%s\n' "$VENCORD_REV" "$hash" > "$marker"
    chmod 644 "$marker"
    if [[ -e "$VENCORD_ROOT" ]]; then
        remove_owned_tree "$MANAGED_ROOT/vencord.previous"
        mv -- "$VENCORD_ROOT" "$MANAGED_ROOT/vencord.previous"
    fi
    mv -- "$stage_v" "$VENCORD_ROOT"
    VENCORD_ROOT="$MANAGED_ROOT/vencord"
    VENCORD_DIST="$VENCORD_ROOT/dist"
}

ensure_managed_vencord() {
    if managed_vencord_valid; then
        info 'Vencord: existing managed build is current.'
        return 0
    fi
    build_managed_vencord
    info "Vencord: managed build ready at $VENCORD_DIST."
}

vencord_plugin_setting_state() {
    [[ -e "$VENCORD_SETTINGS_PATH" ]] || { printf 'missing\n'; return 0; }
    [[ ! -L "$VENCORD_SETTINGS_PATH" && -f "$VENCORD_SETTINGS_PATH" ]] || return 1
    node -e '
const fs = require("fs");
const data = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
if (!data || typeof data !== "object" || Array.isArray(data)) throw new Error("settings root is not an object");
if (data.plugins !== undefined && (!data.plugins || typeof data.plugins !== "object" || Array.isArray(data.plugins))) throw new Error("plugins is not an object");
const plugin = data.plugins?.VesktopVoiceOverlay;
if (plugin !== undefined && (!plugin || typeof plugin !== "object" || Array.isArray(plugin))) throw new Error("plugin settings are not an object");
if (plugin?.enabled !== undefined && typeof plugin.enabled !== "boolean") throw new Error("plugin enabled setting is not boolean");
process.stdout.write(plugin?.enabled === true ? "enabled\n" : plugin?.enabled === false ? "disabled\n" : "missing\n");
' "$VENCORD_SETTINGS_PATH"
}

load_vencord_settings_ownership() {
    if [[ -f "$STATE_FILE" ]]; then
        VENCORD_SETTINGS_CHANGED=$(state_get vencord_settings_changed 2>/dev/null || printf '0')
        VENCORD_SETTINGS_HAD_FILE=$(state_get vencord_settings_had_file 2>/dev/null || printf '0')
        VENCORD_SETTINGS_ORIGINAL_HASH=$(state_get vencord_settings_original_hash 2>/dev/null || true)
        VENCORD_SETTINGS_MANAGED_HASH=$(state_get vencord_settings_managed_hash 2>/dev/null || true)
    fi
}

enable_managed_vencord_plugin() {
    local setting_state rollback tmp mode
    if (( DRY_RUN )); then
        info "[dry-run] enable VesktopVoiceOverlay in $VENCORD_SETTINGS_PATH"
        return 0
    fi
    need_cmd node
    if ! setting_state=$(vencord_plugin_setting_state); then
        die "Vencord settings are malformed; refusing to overwrite: $VENCORD_SETTINGS_PATH"
    fi
    VENCORD_SETTINGS_HAD_FILE=0
    if [[ -e "$VENCORD_SETTINGS_PATH" ]]; then
        [[ ! -L "$VENCORD_SETTINGS_PATH" && -f "$VENCORD_SETTINGS_PATH" ]] ||
            die "Vencord settings path is not a regular file: $VENCORD_SETTINGS_PATH"
        VENCORD_SETTINGS_HAD_FILE=1
    fi
    [[ "$setting_state" == enabled ]] && return 0

    mkdir -p -- "$BACKUP_ROOT"
    if (( VENCORD_SETTINGS_HAD_FILE )); then
        VENCORD_SETTINGS_ORIGINAL_HASH=$(sha256_file "$VENCORD_SETTINGS_PATH")
        rollback=$(mktemp "$MANAGED_ROOT/.settings-rollback.XXXXXX")
        TEMP_PATHS+=("$rollback")
        cp -p -- "$VENCORD_SETTINGS_PATH" "$rollback"
        cp -p -- "$VENCORD_SETTINGS_PATH" "$VENCORD_SETTINGS_BACKUP"
    else
        VENCORD_SETTINGS_ORIGINAL_HASH=""
        VENCORD_SETTINGS_ROLLBACK_PATH=""
        rm -f -- "$VENCORD_SETTINGS_BACKUP"
    fi
    VENCORD_SETTINGS_ROLLBACK_PATH="${rollback:-}"
    mkdir -p -- "$(dirname -- "$VENCORD_SETTINGS_PATH")"
    mode=600
    if (( VENCORD_SETTINGS_HAD_FILE )); then
        mode=$(stat -c '%a' "$VENCORD_SETTINGS_PATH")
    fi
    tmp=$(mktemp "$(dirname -- "$VENCORD_SETTINGS_PATH")/.settings.XXXXXX")
    node - "$VENCORD_SETTINGS_PATH" "$tmp" "$mode" <<'NODE'
const fs = require("fs");
const path = process.argv[2];
const tmp = process.argv[3];
const mode = Number.parseInt(process.argv[4], 8);
let data = {};
if (fs.existsSync(path)) {
    data = JSON.parse(fs.readFileSync(path, "utf8"));
    if (!data || typeof data !== "object" || Array.isArray(data)) throw new Error("settings root is not an object");
}
if (data.plugins !== undefined && (!data.plugins || typeof data.plugins !== "object" || Array.isArray(data.plugins))) throw new Error("plugins is not an object");
if (data.plugins?.VesktopVoiceOverlay !== undefined && (!data.plugins.VesktopVoiceOverlay || typeof data.plugins.VesktopVoiceOverlay !== "object" || Array.isArray(data.plugins.VesktopVoiceOverlay))) throw new Error("plugin settings are not an object");
data.plugins ??= {};
data.plugins.VesktopVoiceOverlay ??= {};
data.plugins.VesktopVoiceOverlay.enabled = true;
fs.writeFileSync(tmp, JSON.stringify(data, null, 4) + "\n", { mode });
fs.chmodSync(tmp, mode);
fs.renameSync(tmp, path);
NODE
    VENCORD_SETTINGS_CHANGED=1
    VENCORD_SETTINGS_MODIFIED_THIS_RUN=1
    VENCORD_SETTINGS_MANAGED_HASH=$(sha256_file "$VENCORD_SETTINGS_PATH")
    [[ "$(vencord_plugin_setting_state)" == enabled ]] || die 'failed to enable VesktopVoiceOverlay'
}

restore_managed_vencord_plugin() {
    local current_hash
    [[ "$VENCORD_SETTINGS_CHANGED" == 1 ]] || return 0
    [[ ! -L "$VENCORD_SETTINGS_PATH" ]] || die "refusing to restore symlinked Vencord settings: $VENCORD_SETTINGS_PATH"
    [[ -f "$VENCORD_SETTINGS_PATH" ]] || { warn 'managed Vencord settings file is missing; leaving it untouched'; return 0; }
    current_hash=$(sha256_file "$VENCORD_SETTINGS_PATH")
    if [[ "$current_hash" != "$VENCORD_SETTINGS_MANAGED_HASH" ]]; then
        warn 'Vencord settings changed after installation; leaving them untouched.'
        return 0
    fi
    if [[ "$VENCORD_SETTINGS_HAD_FILE" == 1 ]]; then
        [[ -f "$VENCORD_SETTINGS_BACKUP" ]] || die 'managed Vencord settings backup is missing'
        cp -p -- "$VENCORD_SETTINGS_BACKUP" "$VENCORD_SETTINGS_PATH"
    else
        rm -f -- "$VENCORD_SETTINGS_PATH"
    fi
}

read_vesktop_vencord_dir() {
    [[ -f "$VESKTOP_STATE_PATH" ]] || return 0
    need_cmd node
    node -e 'const fs=require("fs"); try { const d=JSON.parse(fs.readFileSync(process.argv[1],"utf8")); if (typeof d.vencordDir === "string") process.stdout.write(d.vencordDir); } catch {}' "$VESKTOP_STATE_PATH"
}

set_vesktop_vencord_dir() {
    local path="$1" dist="$2"
    need_cmd node
    mkdir -p -- "$(dirname -- "$path")"
    node - "$path" "$dist" <<'NODE'
const fs = require("fs");
const path = process.argv[2];
const dist = process.argv[3];
let data = {};
let mode = 0o600;
if (fs.existsSync(path)) {
    data = JSON.parse(fs.readFileSync(path, "utf8"));
    mode = fs.statSync(path).mode & 0o777;
}
if (!data || typeof data !== "object" || Array.isArray(data)) throw new Error("Vesktop state is not a JSON object");
data.vencordDir = dist;
const tmp = `${path}.discord-voice-overlay.${process.pid}.tmp`;
fs.writeFileSync(tmp, JSON.stringify(data, null, 4) + "\n", { mode: 0o600 });
fs.chmodSync(tmp, mode);
fs.renameSync(tmp, path);
NODE
}

remove_vesktop_vencord_dir() {
    local path="$1"
    [[ -f "$path" ]] || return 0
    need_cmd node
    node - "$path" <<'NODE'
const fs = require("fs");
const path = process.argv[2];
const data = JSON.parse(fs.readFileSync(path, "utf8"));
if (!data || typeof data !== "object" || Array.isArray(data)) throw new Error("Vesktop state is not a JSON object");
const tmp = `${path}.discord-voice-overlay.${process.pid}.tmp`;
fs.writeFileSync(tmp, JSON.stringify(data, null, 4) + "\n");
fs.renameSync(tmp, path);
NODE
}

backup_vesktop_state() {
    if [[ -f "$VESKTOP_STATE_PATH" ]]; then
        VESKTOP_STATE_HAD_FILE=1
        mkdir -p -- "$BACKUP_ROOT"
        cp -p -- "$VESKTOP_STATE_PATH" "$BACKUP_ROOT/vesktop-state.json"
    else
        VESKTOP_STATE_HAD_FILE=0
    fi
}

preflight_vesktop() {
    local current
    [[ ! -L "$VESKTOP_STATE_PATH" ]] || die "Vesktop state path is a symlink; refusing to replace it: $VESKTOP_STATE_PATH"
    current=$(read_vesktop_vencord_dir || true)
    if [[ -n "$current" && "$current" != "$VENCORD_DIST" ]]; then
        confirm "Existing custom Vencord path detected at $current. Switch Vesktop to the managed build and keep a rollback copy?" ||
            die 'existing custom Vencord was not changed'
    fi
}

configure_vesktop() {
    local current
    VESKTOP_STATE_HAD_FILE=0
    current=$(read_vesktop_vencord_dir || true)
    if [[ -n "$current" && "$current" != "$VENCORD_DIST" ]]; then
        backup_vesktop_state
    elif [[ -f "$VESKTOP_STATE_PATH" ]]; then
        VESKTOP_STATE_HAD_FILE=1
    fi
    if (( DRY_RUN )); then
        info "[dry-run] set $VESKTOP_STATE_PATH vencordDir=$VENCORD_DIST"
        return 0
    fi
    set_vesktop_vencord_dir "$VESKTOP_STATE_PATH" "$VENCORD_DIST"
    info 'Vesktop: managed Vencord path configured.'
}

preflight_discord() {
    local target
    target=$(latest_discord_target || true)
    [[ -n "$target" ]] || die "native Discord data was detected, but no app-* target exists under $CONFIG_HOME/discord"
    if [[ -f "$target/_app.asar" ]]; then
        if [[ "$(state_get client 2>/dev/null || true)" == discord &&
            "$(state_get discord_target 2>/dev/null || true)" == "$target" ]]; then
            return 0
        fi
        die "Discord target already has an existing Vencord injection; refusing to adopt it: $target"
    fi
    confirm "Discord Desktop application resources will be modified through Vencord's official injector at $target. Continue?" ||
        die 'Discord integration was not approved'
}

configure_discord() {
    local target location original
    target=$(latest_discord_target || true)
    [[ -n "$target" ]] || die 'Discord target disappeared before configuration'
    DISCORD_TARGET="$target"
    if [[ -f "$target/_app.asar" ]]; then
        [[ "$(state_get client 2>/dev/null || true)" == discord &&
            "$(state_get discord_target 2>/dev/null || true)" == "$target" ]] ||
            die "refusing to overwrite an existing Discord injection: $target"
        DISCORD_ORIGINAL_HASH="$(state_get discord_original_hash)"
        return 0
    fi
    original="$target/app.asar"
    location="${target%/resources}"
    DISCORD_ORIGINAL_HASH=$(sha256_file "$original")
    if (( DRY_RUN )); then
        info "[dry-run] run official Vencord injector for $target"
        return 0
    fi
    need_cmd node
    if ! (cd "$VENCORD_ROOT" && VENCORD_USER_DATA_DIR="$MANAGED_ROOT/vencord-data" VENCORD_DEV_INSTALL=1 DISCORD_USER_DATA_DIR="$CONFIG_HOME/discord" node scripts/runInstaller.mjs -- --install -location "$location"); then
        warn 'Vencord injection failed; attempting official uninject rollback'
        (cd "$VENCORD_ROOT" && VENCORD_USER_DATA_DIR="$MANAGED_ROOT/vencord-data" VENCORD_DEV_INSTALL=1 DISCORD_USER_DATA_DIR="$CONFIG_HOME/discord" node scripts/runInstaller.mjs -- --uninstall -location "$location") || true
        die 'Discord integration failed'
    fi
    [[ -f "$target/_app.asar" ]] || die 'injector completed without an original app.asar backup'
    [[ -f "$target/app.asar" ]] || die 'injector completed without app.asar'
    [[ "$(sha256_file "$target/_app.asar")" == "$DISCORD_ORIGINAL_HASH" ]] ||
        die 'injector backup does not match the original app.asar'
    info 'Discord Desktop: official Vencord injection completed.'
}

systemd_exec_path() {
    local value="$1"
    require_absolute "$value"
    [[ "$value" != *$'\n'* && "$value" != *$'\r'* ]] ||
        die 'overlay path cannot contain newlines'
    value=${value//\\/\\\\}
    value=${value//"/\\"}
    value=${value//\$/\\\$}
    value=${value//%/%%}
    printf '"%s"\n' "$value"
}

service_content() {
    local escaped
    escaped=$(systemd_exec_path "$OVERLAY_EXECUTABLE")
    printf '[Unit]\nDescription=%s (Wayland layer-shell voice widget)\nPartOf=graphical-session.target\nAfter=graphical-session.target\n\n[Service]\nExecStart=%s\nEnvironment=RUST_BACKTRACE=1\nRestart=always\nRestartSec=2\n\n[Install]\nWantedBy=graphical-session.target\n' "$PRODUCT_NAME" "$escaped"
}

ensure_service() {
    local tmp created=0
    if (( ! DRY_RUN )); then
        mkdir -p -- "$(dirname -- "$SERVICE_PATH")"
    fi
    if [[ -e "$SERVICE_PATH" ]]; then
        [[ ! -L "$SERVICE_PATH" ]] || die "existing service path is a symlink: $SERVICE_PATH"
        [[ -f "$SERVICE_PATH" ]] || die "existing service path is not a regular file: $SERVICE_PATH"
        cmp -s <(service_content) "$SERVICE_PATH" ||
            die "existing user service differs; refusing to overwrite: $SERVICE_PATH"
    elif (( ! DRY_RUN )); then
        tmp=$(mktemp "$(dirname -- "$SERVICE_PATH")/.vesktop-voice-overlay.XXXXXX")
        service_content > "$tmp"
        chmod 644 "$tmp"
        mv -f -- "$tmp" "$SERVICE_PATH"
        created=1
    else
        info "[dry-run] install user service $SERVICE_PATH"
    fi
    if (( ! DRY_RUN )); then
        need_cmd systemctl
        if ! systemctl --user daemon-reload ||
            ! systemctl --user enable --now vesktop-voice-overlay.service; then
            (( created )) && rm -f -- "$SERVICE_PATH"
            die 'systemd user service could not be enabled or started'
        fi
        info 'Service: enabled and started.'
    fi
}

deactivate_previous_client() {
    local old target location original current
    old=$(state_get client 2>/dev/null || true)
    [[ -n "$old" && "$old" != "$SELECTED_CLIENT" ]] || return 0
    info "Switching ownership from $(client_label "$old") to $(client_label "$SELECTED_CLIENT")."
    case "$old" in
        vesktop)
            current=$(read_vesktop_vencord_dir || true)
            [[ "$current" == "$VENCORD_DIST" ]] || die 'previous Vesktop integration is no longer attributable; refusing to remove it'
            if (( ! DRY_RUN )); then
                if [[ -f "$BACKUP_ROOT/vesktop-state.json" ]]; then
                    cp -p -- "$BACKUP_ROOT/vesktop-state.json" "$VESKTOP_STATE_PATH"
                else
                    remove_vesktop_vencord_dir "$VESKTOP_STATE_PATH"
                fi
            fi
            ;;
        discord)
            target=$(state_get discord_target 2>/dev/null || true)
            original=$(state_get discord_original_hash 2>/dev/null || true)
            location="${target%/resources}"
            discord_target_is_safe "$target" || die 'previous Discord target path is unsafe; refusing to guess'
            [[ -n "$target" && -f "$target/_app.asar" ]] || die 'previous Discord target is not still injected; refusing to guess'
            [[ "$(sha256_file "$target/_app.asar")" == "$original" ]] || die 'previous Discord backup changed; refusing to uninject'
            if (( ! DRY_RUN )); then
                need_cmd node
                (cd "$VENCORD_ROOT" && VENCORD_USER_DATA_DIR="$MANAGED_ROOT/vencord-data" VENCORD_DEV_INSTALL=1 DISCORD_USER_DATA_DIR="$CONFIG_HOME/discord" node scripts/runInstaller.mjs -- --uninstall -location "$location")
            fi
            ;;
        *) die "unknown previous client in installer state: $old" ;;
    esac
}

status_service() {
    if [[ ! -f "$SERVICE_PATH" ]]; then
        info 'Service: not installed by this manager.'
        return 0
    fi
    info "Service file: $SERVICE_PATH"
    if command -v systemctl >/dev/null 2>&1; then
        info "Service enabled: $(systemctl --user is-enabled vesktop-voice-overlay.service 2>/dev/null || printf 'no')"
        info "Service active: $(systemctl --user is-active vesktop-voice-overlay.service 2>/dev/null || printf 'no')"
    fi
}

status_cmd() {
    detect_native_clients
    info "$PRODUCT_NAME status"
    info 'Detected clients:'
    print_clients
    if [[ -f "$STATE_FILE" ]]; then
        if state_is_valid; then
            info "Selected client: $(client_label "$(state_get client)")"
            info "Overlay: $(state_get overlay_path) ($(state_get overlay_version))"
            info "Managed Vencord: $(state_get vencord_dist)"
            info "Vencord revision: $(state_get vencord_revision)"
            info "Integration: $(state_get integration_method)"
        else
            info "Installer state: CORRUPT (preserved): $STATE_FILE"
        fi
    else
        info 'Selected client: none'
    fi
    info "Config: $CONFIG_HOME/vesktop-voice-overlay/config.toml (preserved by manager)"
    status_service
}

doctor_cmd() {
    local command
    info "$PRODUCT_NAME doctor"
    if [[ "$(uname -s)" == Linux ]]; then
        info 'OS: Linux'
    else
        warn 'OS is not Linux'
    fi
    for command in git pnpm node curl sha256sum realpath systemctl; do
        if command -v "$command" >/dev/null 2>&1; then info "  $command: $(command -v "$command")"; else warn "  missing command: $command"; fi
    done
    if [[ "${XDG_SESSION_TYPE:-}" == wayland ]]; then info 'Session: Wayland'; else warn 'Session is not reported as Wayland'; fi
    detect_native_clients
    info 'Detected clients:'
    print_clients
    if [[ "$CLIENT_DISCORD_FLATPAK" -eq 1 || "$CLIENT_VESKTOP_FLATPAK" -eq 1 ]]; then
        warn 'Flatpak clients are detected but unsupported by this candidate'
    fi
}

install_cmd() {
    require_user
    detect_native_clients
    select_client
    ensure_state_usable
    load_vencord_settings_ownership
    if [[ "$SELECTED_CLIENT" == vesktop ]]; then preflight_vesktop; else preflight_discord; fi
    ensure_overlay
    ensure_managed_vencord
    enable_managed_vencord_plugin
    ensure_service
    if [[ "$SELECTED_CLIENT" == vesktop ]]; then configure_vesktop; else configure_discord; fi
    deactivate_previous_client
    write_state
    INSTALL_COMPLETED=1
    info "Ready: $(client_label "$SELECTED_CLIENT") is configured for $PRODUCT_NAME."
}

update_cmd() {
    ensure_state_usable
    if [[ -f "$STATE_FILE" ]]; then
        SELECTED_CLIENT=$(state_get client) || die 'installer state has no selected client'
        [[ "$SELECTED_CLIENT" == vesktop || "$SELECTED_CLIENT" == discord ]] || die 'installer state has invalid client'
        REQUESTED_CLIENT="$SELECTED_CLIENT"
    fi
    install_cmd
}

repair_cmd() {
    ensure_state_usable
    [[ -f "$STATE_FILE" ]] || die 'no installer state exists; run install first'
    SELECTED_CLIENT=$(state_get client) || die 'installer state has no selected client'
    REQUESTED_CLIENT="$SELECTED_CLIENT"
    install_cmd
}

remove_service() {
    [[ -e "$SERVICE_PATH" ]] || return 0
    [[ ! -L "$SERVICE_PATH" ]] || die "refusing to remove symlink service path: $SERVICE_PATH"
    [[ -f "$SERVICE_PATH" ]] || die "refusing to remove non-file service path: $SERVICE_PATH"
    cmp -s <(service_content) "$SERVICE_PATH" || {
        warn "leaving foreign or modified service untouched: $SERVICE_PATH"
        return 1
    }
    if (( ! DRY_RUN )); then
        systemctl --user disable --now vesktop-voice-overlay.service 2>/dev/null || true
        systemctl --user daemon-reload || true
        rm -f -- "$SERVICE_PATH"
    else
        info "[dry-run] remove managed user service $SERVICE_PATH"
    fi
}

uninstall_cmd() {
    local client target location original current
    require_user
    ensure_state_usable
    [[ -f "$STATE_FILE" ]] || { info 'No installer state found; nothing owned to uninstall.'; return 0; }
    state_is_valid || die 'installer state is corrupt; refusing uninstall'
    load_vencord_settings_ownership
    client=$(state_get client)
    OVERLAY_EXECUTABLE=$(state_get overlay_path)
    [[ "$OVERLAY_EXECUTABLE" == "$MANAGED_ROOT/"* || "$OVERLAY_EXECUTABLE" == /usr/bin/vesktop-voice-overlay || "$OVERLAY_EXECUTABLE" == "$HOME/.local/bin/vesktop-voice-overlay" ]] ||
        die 'state overlay path is outside approved ownership locations'
    if [[ "$client" == vesktop ]]; then
        current=$(read_vesktop_vencord_dir || true)
        if [[ "$current" == "$VENCORD_DIST" ]]; then
            if (( ! DRY_RUN )); then
                if [[ -f "$BACKUP_ROOT/vesktop-state.json" ]]; then
                    cp -p -- "$BACKUP_ROOT/vesktop-state.json" "$VESKTOP_STATE_PATH"
                else
                    remove_vesktop_vencord_dir "$VESKTOP_STATE_PATH"
                fi
            fi
        elif [[ -n "$current" ]]; then
            warn 'Vesktop state changed after installation; preserving it and leaving integration for manual review.'
        fi
    elif [[ "$client" == discord ]]; then
        target=$(state_get discord_target 2>/dev/null || true)
        original=$(state_get discord_original_hash 2>/dev/null || true)
        location="${target%/resources}"
        discord_target_is_safe "$target" || die 'stored Discord target path is unsafe; refusing uninject'
        if [[ -n "$target" && -f "$target/_app.asar" ]]; then
            [[ "$(sha256_file "$target/_app.asar")" == "$original" ]] ||
                die 'Discord original backup changed; refusing uninject'
            if (( ! DRY_RUN )); then
                need_cmd pnpm
                need_cmd node
                (cd "$VENCORD_ROOT" && VENCORD_USER_DATA_DIR="$MANAGED_ROOT/vencord-data" VENCORD_DEV_INSTALL=1 DISCORD_USER_DATA_DIR="$CONFIG_HOME/discord" node scripts/runInstaller.mjs -- --uninstall -location "$location")
            fi
        else
            warn 'Discord injection is no longer present; leaving application resources untouched.'
        fi
    else
        die "unknown client in installer state: $client"
    fi
    restore_managed_vencord_plugin
    remove_service || die 'uninstall stopped because the service was not manager-owned'
    remove_owned_tree "$MANAGED_ROOT"
    if (( ! DRY_RUN )); then
        rm -f -- "$STATE_FILE"
        rmdir -- "$STATE_DIR" 2>/dev/null || true
    fi
    info 'Uninstall complete. The overlay config file was preserved.'
}

parse_args() {
    local command='install'
    while (($#)); do
        case "$1" in
            install|update|repair|status|doctor|uninstall) command="$1" ;;
            --client)
                (($# >= 2)) || die '--client requires vesktop or discord'
                REQUESTED_CLIENT="$2"
                [[ "$REQUESTED_CLIENT" == vesktop || "$REQUESTED_CLIENT" == discord ]] || die 'client must be vesktop or discord'
                shift
                ;;
            --yes) ASSUME_YES=1 ;;
            --dry-run) DRY_RUN=1 ;;
            --version) version; exit 0 ;;
            --help|-h) usage; exit 0 ;;
            --) shift; (($# == 0)) || die 'unexpected arguments'; break ;;
            -*) die "unknown option: $1" ;;
            *) die "unknown command or argument: $1" ;;
        esac
        shift
    done
    case "$command" in
        install) install_cmd ;;
        update) update_cmd ;;
        repair) repair_cmd ;;
        status) status_cmd ;;
        doctor) doctor_cmd ;;
        uninstall) uninstall_cmd ;;
    esac
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
    parse_args "$@"
fi
