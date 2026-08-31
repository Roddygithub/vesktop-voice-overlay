#!/usr/bin/env bash
set -Eeuo pipefail
IFS=$'\n\t'

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
INSTALLER="$SCRIPT_DIR/install.sh"
TEST_ROOT=$(mktemp -d "${TMPDIR:-/tmp}/discord-voice-overlay-tests.XXXXXX")
MOCK_BIN="$TEST_ROOT/bin"
mkdir -p -- "$MOCK_BIN"
trap 'rm -rf -- "$TEST_ROOT"' EXIT

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
pass() { printf 'PASS: %s\n' "$*"; }
assert_file() { [[ -f "$1" ]] || fail "missing file: $1"; }
assert_not_file() { [[ ! -e "$1" && ! -L "$1" ]] || fail "unexpected file: $1"; }
assert_contains() { grep -Fq -- "$1" "$2" || fail "missing '$1' in $2"; }
assert_not_contains() { ! grep -Fq -- "$1" "$2" || fail "unexpected '$1' in $2"; }

REAL_NODE=$(command -v node) || fail 'node is required for installer fixtures'
export DVO_REAL_NODE="$REAL_NODE"

cat > "$MOCK_BIN/pnpm" <<'EOF'
#!/usr/bin/env bash
set -Eeuo pipefail
printf '%s\n' "$*" >> "$DVO_CALL_LOG"
case "${1:-}" in
    install) exit 0 ;;
    build)
        [[ "${DVO_FAIL_BUILD:-0}" == 1 ]] && exit 42
        mkdir -p dist
        printf 'VesktopVoiceOverlay\n' > dist/vencordDesktopMain.js
        printf 'VesktopVoiceOverlay\n' > dist/vencordDesktopRenderer.js
        ;;
    *) exit 0 ;;
esac
EOF
chmod 755 "$MOCK_BIN/pnpm"

cat > "$MOCK_BIN/node" <<'EOF'
#!/usr/bin/env bash
set -Eeuo pipefail
if [[ "${1:-}" == scripts/runInstaller.mjs ]]; then
    printf 'node %s\n' "$*" >> "$DVO_CALL_LOG"
    args=("$@")
    location=''
    for ((i = 0; i < ${#args[@]}; i++)); do
        if [[ "${args[i]}" == -location && $((i + 1)) -lt ${#args[@]} ]]; then
            location="${args[i + 1]}"
        fi
    done
    [[ -n "$location" ]] || exit 46
    [[ "$location" == "$HOME/.config/discord"/app-* ]] || exit 47
    resources="${location%/}/resources"
    if [[ " $* " == *' --uninstall '* ]]; then
        [[ -f "$resources/_app.asar" ]] || exit 44
        mv -- "$resources/_app.asar" "$resources/app.asar"
        exit 0
    fi
    [[ "${DVO_FAIL_INJECT:-0}" == 1 ]] && exit 43
    [[ -f "$resources/app.asar" ]] || exit 44
    mv -- "$resources/app.asar" "$resources/_app.asar"
    printf 'managed injector\n' > "$resources/app.asar"
    exit 0
fi
exec "$DVO_REAL_NODE" "$@"
EOF
chmod 755 "$MOCK_BIN/node"

cat > "$MOCK_BIN/systemctl" <<'EOF'
#!/usr/bin/env bash
set -Eeuo pipefail
printf 'systemctl %s\n' "$*" >> "$DVO_CALL_LOG"
[[ "${DVO_FAIL_SYSTEMCTL:-0}" == 1 ]] && exit 45
args=("$@")
case "${args[1]:-}" in
    is-enabled) printf 'enabled\n' ;;
    is-active) printf 'active\n' ;;
esac
EOF
chmod 755 "$MOCK_BIN/systemctl"

cat > "$MOCK_BIN/systemd-escape" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "${2:?path missing}"
EOF
chmod 755 "$MOCK_BIN/systemd-escape"

make_fixture() {
    local client="$1" state_mode="${2:-fresh}"
    CASE_ROOT=$(mktemp -d "$TEST_ROOT/case.XXXXXX")
    CASE_HOME="$CASE_ROOT/home"
    CASE_SYSTEM="$CASE_ROOT/system"
    MOCK_LOG="$CASE_ROOT/calls.log"
    STATE_FILE_FIXTURE="$CASE_HOME/.local/state/discord-voice-overlay/state.env"
    VENCORD_SETTINGS_FIXTURE="$CASE_HOME/.config/Vencord/settings/settings.json"
    VENCORD_SETTINGS_BEFORE="$CASE_ROOT/vencord-settings.before"
    mkdir -p -- "$CASE_HOME/.config" "$CASE_HOME/.local/share" "$CASE_HOME/.local/state" \
        "$CASE_SYSTEM/usr/bin" "$CASE_SYSTEM/usr/lib" "$CASE_SYSTEM/usr/share" \
        "$(dirname -- "$VENCORD_SETTINGS_FIXTURE")"
    : > "$MOCK_LOG"
    cat > "$VENCORD_SETTINGS_FIXTURE" <<'EOF'
{
    "uiElements": {"keep": true},
    "plugins": {
        "UnrelatedEnabled": {"enabled": true, "custom": "keep"},
        "UnrelatedDisabled": {"enabled": false, "custom": "keep"},
        "VesktopVoiceOverlay": {"enabled": false, "custom": "keep"}
    }
}
EOF
    cp -p "$VENCORD_SETTINGS_FIXTURE" "$VENCORD_SETTINGS_BEFORE"
    if [[ "$client" == vesktop || "$client" == both ]]; then
        printf '#!/bin/sh\n' > "$CASE_SYSTEM/usr/bin/vesktop"
        chmod 755 "$CASE_SYSTEM/usr/bin/vesktop"
        mkdir -p -- "$CASE_SYSTEM/usr/lib/vesktop"
    fi
    if [[ "$client" == discord || "$client" == both ]]; then
        printf '#!/bin/sh\n' > "$CASE_SYSTEM/usr/bin/discord"
        chmod 755 "$CASE_SYSTEM/usr/bin/discord"
        mkdir -p -- "$CASE_SYSTEM/usr/share/discord"
        mkdir -p -- "$CASE_HOME/.config/discord/app-1.0.0/resources"
        printf 'original discord app\n' > "$CASE_HOME/.config/discord/app-1.0.0/resources/app.asar"
    fi
    if [[ "$state_mode" == custom-vesktop ]]; then
        mkdir -p -- "$CASE_HOME/custom-vencord/dist"
        printf '{"vencordDir":"%s"}\n' "$CASE_HOME/custom-vencord/dist" > "$CASE_HOME/.config/vesktop-state.json"
        mkdir -p -- "$CASE_HOME/.config/vesktop"
        mv -- "$CASE_HOME/.config/vesktop-state.json" "$CASE_HOME/.config/vesktop/state.json"
    elif [[ "$client" == vesktop || "$client" == both ]]; then
        mkdir -p -- "$CASE_HOME/.config/vesktop"
        printf '{"windowBounds":{"width":900}}\n' > "$CASE_HOME/.config/vesktop/state.json"
    fi
    if [[ "$state_mode" == injected-discord ]]; then
        mv -- "$CASE_HOME/.config/discord/app-1.0.0/resources/app.asar" \
            "$CASE_HOME/.config/discord/app-1.0.0/resources/_app.asar"
        printf 'existing injector\n' > "$CASE_HOME/.config/discord/app-1.0.0/resources/app.asar"
    fi
    if [[ "$state_mode" == flatpak ]]; then
        cat > "$MOCK_BIN/flatpak" <<'EOF'
#!/usr/bin/env bash
printf 'com.discordapp.Discord\n'
EOF
        chmod 755 "$MOCK_BIN/flatpak"
    else
        rm -f -- "$MOCK_BIN/flatpak"
    fi
    mkdir -p -- "$CASE_ROOT/vencord-source/src"
    printf '{"name":"vencord","private":true}\n' > "$CASE_ROOT/vencord-source/package.json"
    printf 'dist/\n' > "$CASE_ROOT/vencord-source/.gitignore"
    git -C "$CASE_ROOT/vencord-source" init -q
    git -C "$CASE_ROOT/vencord-source" config user.email test@example.invalid
    git -C "$CASE_ROOT/vencord-source" config user.name test
    git -C "$CASE_ROOT/vencord-source" add package.json .gitignore
    git -C "$CASE_ROOT/vencord-source" commit -qm fixture
    FAKE_VENCORD_REV=$(git -C "$CASE_ROOT/vencord-source" rev-parse HEAD)
    cat > "$CASE_ROOT/overlay" <<'EOF'
#!/usr/bin/env bash
[[ "${1:-}" == --version ]] && printf 'vesktop-voice-overlay 1.3.0\n'
EOF
    chmod 755 "$CASE_ROOT/overlay"
    export CASE_HOME CASE_SYSTEM MOCK_LOG FAKE_VENCORD_REV STATE_FILE_FIXTURE \
        VENCORD_SETTINGS_FIXTURE VENCORD_SETTINGS_BEFORE
    export DVO_CALL_LOG="$MOCK_LOG"
    export PATH="$MOCK_BIN:/usr/bin:$PATH"
    export HOME="$CASE_HOME"
    export XDG_CONFIG_HOME="$CASE_HOME/.config"
    export XDG_DATA_HOME="$CASE_HOME/.local/share"
    export XDG_STATE_HOME="$CASE_HOME/.local/state"
    export DVO_SYSTEM_ROOT="$CASE_SYSTEM"
    export DVO_MANAGED_ROOT="$CASE_HOME/.local/share/discord-voice-overlay"
    export DVO_OVERLAY_BINARY="$CASE_ROOT/overlay"
    export DVO_VENCORD_REPO="$CASE_ROOT/vencord-source"
    export DVO_VENCORD_REV="$FAKE_VENCORD_REV"
    export DVO_TEST_MODE=1
    unset DVO_FAIL_BUILD DVO_FAIL_INJECT DVO_FAIL_SYSTEMCTL
}

run_installer() {
    "$INSTALLER" "$@"
}

assert_plugin_enabled() {
    node -e 'const fs=require("fs"),d=JSON.parse(fs.readFileSync(process.argv[1],"utf8")); if (d.plugins?.VesktopVoiceOverlay?.enabled !== true) process.exit(1)' "$1" ||
        fail "VesktopVoiceOverlay is not enabled in $1"
}

assert_plugin_settings_preserved() {
    node -e 'const fs=require("fs"),d=JSON.parse(fs.readFileSync(process.argv[1],"utf8")); const p=d.plugins||{}; if (!d.uiElements?.keep || p.UnrelatedEnabled?.enabled !== true || p.UnrelatedEnabled?.custom !== "keep" || p.UnrelatedDisabled?.enabled !== false || p.UnrelatedDisabled?.custom !== "keep" || p.VesktopVoiceOverlay?.custom !== "keep") process.exit(1)' "$1" ||
        fail "unrelated or custom plugin settings changed in $1"
}

case_no_client() {
    make_fixture none
    run_installer status > "$CASE_ROOT/status" 2>&1
    assert_contains 'none detected' "$CASE_ROOT/status"
    if run_installer install > "$CASE_ROOT/install" 2>&1; then fail 'no-client install unexpectedly succeeded'; fi
    pass 'no supported client fails safely'
}

case_invalid_overrides() {
    if DVO_OVERLAY_VERSION=invalid run_installer --help > "$TEST_ROOT/invalid-version" 2>&1; then
        fail 'invalid overlay version unexpectedly passed'
    fi
    assert_contains 'invalid overlay version' "$TEST_ROOT/invalid-version"
    pass 'invalid installer overrides fail with actionable errors'
}

case_vesktop_lifecycle() {
    make_fixture vesktop
    mkdir -p -- "$CASE_HOME/.config/vesktop-voice-overlay"
    printf 'keep this config\n' > "$CASE_HOME/.config/vesktop-voice-overlay/config.toml"
    run_installer install --client vesktop --yes
    assert_file "$STATE_FILE_FIXTURE"
    assert_file "$DVO_MANAGED_ROOT/vencord/dist/vencordDesktopMain.js"
    assert_plugin_enabled "$VENCORD_SETTINGS_FIXTURE"
    assert_plugin_settings_preserved "$VENCORD_SETTINGS_FIXTURE"
    assert_contains "ExecStart=\"$DVO_MANAGED_ROOT/current/vesktop-voice-overlay\"" "$CASE_HOME/.config/systemd/user/vesktop-voice-overlay.service"
    if command -v systemd-analyze >/dev/null 2>&1; then
        systemd-analyze verify "$CASE_HOME/.config/systemd/user/vesktop-voice-overlay.service" ||
            fail 'generated regular-path service failed systemd verification'
        mkdir -p -- "$CASE_ROOT/path with spaces"
        cp -- "$CASE_ROOT/overlay" "$CASE_ROOT/path with spaces/overlay"
        space_unit="$CASE_ROOT/space.service"
        (
            # shellcheck source=install.sh
            source "$INSTALLER"
            OVERLAY_EXECUTABLE="$CASE_ROOT/path with spaces/overlay"
            export OVERLAY_EXECUTABLE
            service_content
        ) > "$space_unit"
        assert_contains "ExecStart=\"$CASE_ROOT/path with spaces/overlay\"" "$space_unit"
        systemd-analyze verify "$space_unit" || fail 'generated spaced-path service failed systemd verification'
    fi
    assert_contains 'vencordDir' "$CASE_HOME/.config/vesktop/state.json"
    assert_contains 'keep this config' "$CASE_HOME/.config/vesktop-voice-overlay/config.toml"
    builds_before=$(grep -Fc 'build' "$MOCK_LOG" || true)
    settings_after_install=$(sha256sum "$VENCORD_SETTINGS_FIXTURE" | cut -d' ' -f1)
    run_installer install --client vesktop --yes
    [[ "$(sha256sum "$VENCORD_SETTINGS_FIXTURE" | cut -d' ' -f1)" == "$settings_after_install" ]] ||
        fail 'repeat install rewrote Vencord settings'
    run_installer update
    node -e 'const fs=require("fs"),p=process.argv[1],d=JSON.parse(fs.readFileSync(p,"utf8")); d.plugins.VesktopVoiceOverlay.enabled=false; fs.writeFileSync(p,JSON.stringify(d,null,4)+"\n")' "$VENCORD_SETTINGS_FIXTURE"
    run_installer repair
    assert_plugin_enabled "$VENCORD_SETTINGS_FIXTURE"
    builds_after=$(grep -Fc 'build' "$MOCK_LOG" || true)
    [[ "$builds_before" == "$builds_after" ]] || fail 'idempotent setup rebuilt Vencord unexpectedly'
    run_installer uninstall
    assert_not_file "$STATE_FILE_FIXTURE"
    assert_plugin_settings_preserved "$VENCORD_SETTINGS_FIXTURE"
    node -e 'const fs=require("fs"),d=JSON.parse(fs.readFileSync(process.argv[1],"utf8")); if (d.plugins.VesktopVoiceOverlay.enabled !== false) process.exit(1)' "$VENCORD_SETTINGS_FIXTURE" ||
        fail 'uninstall did not restore the pre-repair plugin state'
    assert_contains 'windowBounds' "$CASE_HOME/.config/vesktop/state.json"
    assert_contains 'keep this config' "$CASE_HOME/.config/vesktop-voice-overlay/config.toml"
    run_installer uninstall
    pass 'Vesktop install, repeat, update, repair, uninstall, and config preservation'
}

case_discord_lifecycle() {
    make_fixture discord
    run_installer install --client discord --yes
    assert_file "$CASE_HOME/.config/discord/app-1.0.0/resources/_app.asar"
    assert_plugin_enabled "$VENCORD_SETTINGS_FIXTURE"
    assert_plugin_settings_preserved "$VENCORD_SETTINGS_FIXTURE"
    assert_contains 'node scripts/runInstaller.mjs' "$MOCK_LOG"
    assert_contains "node scripts/runInstaller.mjs -- --install -location $CASE_HOME/.config/discord/app-1.0.0" "$MOCK_LOG"
    assert_not_contains 'pnpm inject' "$MOCK_LOG"
    run_installer status > "$CASE_ROOT/status" 2>&1
    assert_contains 'Discord Desktop' "$CASE_ROOT/status"
    run_installer uninstall
    assert_not_file "$CASE_HOME/.config/discord/app-1.0.0/resources/_app.asar"
    assert_file "$CASE_HOME/.config/discord/app-1.0.0/resources/app.asar"
    cmp -s "$VENCORD_SETTINGS_FIXTURE" "$VENCORD_SETTINGS_BEFORE" || fail 'Discord uninstall changed Vencord settings'
    run_installer install --client discord --yes
    assert_plugin_enabled "$VENCORD_SETTINGS_FIXTURE"
    pass 'Discord official inject/uninject ownership flow'
}

case_discord_injection_failure() {
    make_fixture discord
    export DVO_FAIL_INJECT=1
    if run_installer install --client discord --yes > "$CASE_ROOT/install" 2>&1; then fail 'Discord injection failure unexpectedly succeeded'; fi
    assert_not_file "$STATE_FILE_FIXTURE"
    pass 'Discord injection failures do not publish installer state'
}

case_missing_plugin_settings() {
    make_fixture discord
    node -e 'const fs=require("fs"),p=process.argv[1],d=JSON.parse(fs.readFileSync(p,"utf8")); delete d.plugins.VesktopVoiceOverlay; fs.writeFileSync(p,JSON.stringify(d,null,4)+"\n")' "$VENCORD_SETTINGS_FIXTURE"
    cp -p "$VENCORD_SETTINGS_FIXTURE" "$VENCORD_SETTINGS_BEFORE"
    run_installer install --client discord --yes
    assert_plugin_enabled "$VENCORD_SETTINGS_FIXTURE"
    run_installer uninstall
    cmp -s "$VENCORD_SETTINGS_FIXTURE" "$VENCORD_SETTINGS_BEFORE" || fail 'missing plugin entry was not restored'
    pass 'missing plugin settings are enabled and restored safely'
}

case_malformed_plugin_settings() {
    make_fixture discord
    printf '{ malformed\n' > "$VENCORD_SETTINGS_FIXTURE"
    cp -p "$VENCORD_SETTINGS_FIXTURE" "$VENCORD_SETTINGS_BEFORE"
    if run_installer install --client discord --yes > "$CASE_ROOT/install" 2>&1; then fail 'malformed Vencord settings unexpectedly succeeded'; fi
    cmp -s "$VENCORD_SETTINGS_FIXTURE" "$VENCORD_SETTINGS_BEFORE" || fail 'malformed Vencord settings were overwritten'
    assert_not_file "$STATE_FILE_FIXTURE"
    pass 'malformed Vencord settings fail without overwrite'
}

case_both_requires_selection() {
    make_fixture both
    if run_installer install > "$CASE_ROOT/install" 2>&1; then fail 'both-client install unexpectedly selected a host'; fi
    assert_contains 'select one' "$CASE_ROOT/install"
    run_installer install --client vesktop --yes
    run_installer update
    pass 'both clients require explicit selection'
}

case_existing_conflicts() {
    make_fixture vesktop custom-vesktop
    if run_installer install --client vesktop > "$CASE_ROOT/install" 2>&1; then fail 'custom Vesktop setup unexpectedly replaced'; fi
    assert_contains 'Existing custom Vencord' "$CASE_ROOT/install"
    run_installer install --client vesktop --yes
    run_installer uninstall
    assert_contains 'custom-vencord/dist' "$CASE_HOME/.config/vesktop/state.json"

    make_fixture discord injected-discord
    if run_installer install --client discord --yes > "$CASE_ROOT/install" 2>&1; then fail 'existing Discord injection unexpectedly adopted'; fi
    assert_contains 'existing Vencord injection' "$CASE_ROOT/install"
    pass 'existing custom integrations are never silently adopted'
}

case_unsupported_and_corrupt_state() {
    make_fixture none flatpak
    if run_installer install > "$CASE_ROOT/install" 2>&1; then fail 'Flatpak install unexpectedly succeeded'; fi
    assert_contains 'unsupported' "$CASE_ROOT/install"

    make_fixture vesktop
    mkdir -p -- "$DVO_MANAGED_ROOT"
    mkdir -p -- "$(dirname -- "$STATE_FILE_FIXTURE")"
    printf 'not a state file\n' > "$STATE_FILE_FIXTURE"
    run_installer status > "$CASE_ROOT/status" 2>&1
    assert_contains 'CORRUPT' "$CASE_ROOT/status"
    if run_installer uninstall > "$CASE_ROOT/uninstall" 2>&1; then fail 'corrupt-state uninstall unexpectedly succeeded'; fi
    assert_file "$STATE_FILE_FIXTURE"
    pass 'unsupported client and corrupt state fail safely'
}

case_failure_paths() {
    make_fixture vesktop
    export DVO_FAIL_BUILD=1
    if run_installer install --client vesktop --yes > "$CASE_ROOT/build-failure" 2>&1; then fail 'build failure unexpectedly succeeded'; fi
    assert_not_file "$STATE_FILE_FIXTURE"
    assert_not_file "$DVO_MANAGED_ROOT/vencord"

    make_fixture vesktop
    export DVO_FAIL_SYSTEMCTL=1
    if run_installer install --client vesktop --yes > "$CASE_ROOT/service-failure" 2>&1; then fail 'service failure unexpectedly succeeded'; fi
    assert_not_file "$STATE_FILE_FIXTURE"
    pass 'build and service failures do not publish installer state'
}

case_checksum_and_path_guards() {
    make_fixture none
    printf 'content\n' > "$CASE_ROOT/file"
    printf '%064d  file\n' 0 > "$CASE_ROOT/SHA256SUMS"
    if env HOME="$CASE_HOME" DVO_MANAGED_ROOT="$DVO_MANAGED_ROOT" bash -c \
        "source \"\$1\"; verify_checksum \"\$2\" \"\$3\" file" bash "$INSTALLER" "$CASE_ROOT/file" "$CASE_ROOT/SHA256SUMS"; then
        fail 'invalid checksum unexpectedly passed'
    fi
    if env HOME="$CASE_HOME" DVO_MANAGED_ROOT="$DVO_MANAGED_ROOT" bash -c \
        "source \"\$1\"; remove_owned_tree \"\$2\"" bash "$INSTALLER" "$CASE_HOME"; then
        fail 'unsafe path unexpectedly passed ownership guard'
    fi
    pass 'checksum and recursive-deletion guards fail closed'
}

case_no_client
case_invalid_overrides
case_vesktop_lifecycle
case_discord_lifecycle
case_discord_injection_failure
case_missing_plugin_settings
case_malformed_plugin_settings
case_both_requires_selection
case_existing_conflicts
case_unsupported_and_corrupt_state
case_failure_paths
case_checksum_and_path_guards
printf 'All installer fixture tests passed.\n'
