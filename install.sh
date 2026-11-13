#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -euo pipefail

usage() {
    cat <<'EOF'
Usage: ./install.sh [--replace-conflicts] [--system-greeter]

  --replace-conflicts  Comment out recognized competing Hyprland startup lines
                       (Waybar/Eww/AGS/Quickshell/HyprPanel, alternate
                       notification daemons, wallpaper daemons, and Wofi).
  --system-greeter     Back up and replace /etc/greetd/config.toml. Requires
                       sudo and a bundled greetd/config.toml (not currently shipped).
  -h, --help           Show this help.

The installer builds Amane, installs its user configuration, and reports
likely conflicts. It does not uninstall packages or disable system services.
EOF
}

REPLACE_CONFLICTS=0
SYSTEM_GREETER=0
for arg in "$@"; do
    case "$arg" in
        --replace-conflicts) REPLACE_CONFLICTS=1 ;;
        --system-greeter) SYSTEM_GREETER=1 ;;
        -h|--help) usage; exit 0 ;;
        *) printf 'Unknown option: %s\n' "$arg" >&2; usage >&2; exit 2 ;;
    esac
done

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

# Validate optional system-level inputs before building or modifying user files.
if (( SYSTEM_GREETER )) && [[ ! -f "$ROOT/greetd/config.toml" ]]; then
    printf 'The --system-greeter option is unavailable: greetd/config.toml is not included in this checkout. No files were changed.\n' >&2
    exit 1
fi

CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
STATE_HOME="${XDG_STATE_HOME:-$HOME/.local/state}"
BIN_HOME="${XDG_BIN_HOME:-$HOME/.local/bin}"
BACKUP_ROOT="$STATE_HOME/goad-os-backups/$(date +%Y%m%d-%H%M%S)-$$"
HYPR_HOME="$CONFIG_HOME/hypr"
AMANE_HOME="$CONFIG_HOME/amane"

if ! command -v cargo >/dev/null 2>&1; then
    printf 'Rust and Cargo are required to build Amane. Install them, then rerun this installer.\n' >&2
    exit 1
fi

backup_file() {
    local target="$1" key="$2" saved
    saved="$BACKUP_ROOT/$key"
    [[ -e "$target" || -L "$target" ]] || return 0
    [[ -e "$saved" || -L "$saved" ]] && return 0
    mkdir -p "$(dirname -- "$saved")"
    cp -a -- "$target" "$saved"
    printf '  backup: %s\n' "$saved"
}

install_file() {
    local source="$1" target="$2" key="$3" mode=644
    [[ -f "$target" ]] && cmp -s -- "$source" "$target" && return 0
    if [[ -e "$target" || -L "$target" ]]; then backup_file "$target" "$key"; fi
    [[ -x "$source" ]] && mode=755
    install -D -m "$mode" -- "$source" "$target"
    printf '  installed: %s\n' "$target"
}

install_tree() {
    local source_dir="$1" target_dir="$2" key_root="$3" source rel
    [[ -d "$source_dir" ]] || return 0
    while IFS= read -r -d '' source; do
        rel="${source#"$source_dir"/}"
        install_file "$source" "$target_dir/$rel" "$key_root/$rel"
    done < <(find "$source_dir" -type f -print0)
}

printf 'Building Amane...\n'
cargo build --release --manifest-path "$ROOT/Cargo.toml"
install_file "$ROOT/target/release/amane-shell" "$BIN_HOME/amane-shell" 'local-bin/amane-shell'

printf 'Installing Amane user configuration...\n'
for dir in scripts rofi kitty; do
    install_tree "$ROOT/$dir" "$AMANE_HOME/$dir" "config/amane/$dir"
done
for file in mako.conf mako-red.conf; do
    install_file "$ROOT/$file" "$AMANE_HOME/$file" "config/amane/$file"
done

# Kitty includes the active palette after its other color settings so the
# selected Amane colors override a user's previous palette without replacing
# their unrelated Kitty preferences.
KITTY_HOME="$CONFIG_HOME/kitty"
KITTY_CONFIG="$KITTY_HOME/kitty.conf"
mkdir -p "$KITTY_HOME"
if [[ ! -e "$KITTY_CONFIG" && ! -L "$KITTY_CONFIG" ]]; then
    install_file "$ROOT/kitty/kitty.conf" "$KITTY_CONFIG" 'config/kitty/kitty.conf'
elif ! grep -Fqx 'include amane-theme.conf' "$KITTY_CONFIG"; then
    backup_file "$KITTY_CONFIG" 'config/kitty/kitty.conf'
    printf '\n# Amane selected color palette\ninclude amane-theme.conf\n' >>"$KITTY_CONFIG"
    printf '  enabled Amane color themes in %s\n' "$KITTY_CONFIG"
fi

# Optional themed configs are installed alongside the app so existing user
# defaults are left alone unless the user explicitly chooses them later.
install_tree "$ROOT/fastfetch" "$CONFIG_HOME/fastfetch" 'config/fastfetch'
tmp="$(mktemp)"
sed "s#~/.config/fastfetch/goad-os.txt#$CONFIG_HOME/fastfetch/goad-os.txt#" "$ROOT/fastfetch/config.jsonc" >"$tmp"
install_file "$tmp" "$CONFIG_HOME/fastfetch/config.jsonc" 'config/fastfetch/config.jsonc'
rm -f -- "$tmp"
install_tree "$ROOT/hyprfm/themes" "$CONFIG_HOME/hyprfm/themes" 'config/hyprfm/themes'
install_tree "$ROOT/qt5ct/colors" "$CONFIG_HOME/qt5ct/colors" 'config/qt5ct/colors'
install_tree "$ROOT/qt6ct/colors" "$CONFIG_HOME/qt6ct/colors" 'config/qt6ct/colors'
for version in 5 6; do
    qt_file="$CONFIG_HOME/qt${version}ct/qt${version}ct.conf"
    tmp="$(mktemp)"
    sed "s#^color_scheme_path=.*#color_scheme_path=$CONFIG_HOME/qt${version}ct/colors/amane-teal.conf#" "$ROOT/qt${version}ct/qt${version}ct.conf" >"$tmp"
    install_file "$tmp" "$qt_file" "config/qt${version}ct/qt${version}ct.conf"
    rm -f -- "$tmp"
done

mkdir -p "$HYPR_HOME"
install_tree "$ROOT/hypr" "$HYPR_HOME" 'config/hypr'

if [[ -f "$HYPR_HOME/hyprland.lua" ]]; then
    if ! grep -Fq 'require("deep-current")' "$HYPR_HOME/hyprland.lua"; then
        backup_file "$HYPR_HOME/hyprland.lua" 'config/hypr/hyprland.lua'
        printf '\n-- Goad OS Amane integration\nrequire("deep-current")\n' >>"$HYPR_HOME/hyprland.lua"
        printf '  enabled Amane integration in %s\n' "$HYPR_HOME/hyprland.lua"
    fi
else
    printf '  note: Hyprland Lua config not found; add require("deep-current") to %s/hyprland.lua to enable the bar integration.\n' "$HYPR_HOME"
fi

scan_conflicts() {
    local found=0 tool
    printf '\nChecking common competing tools...\n'
    for tool in waybar eww ags quickshell hyprpanel dunst swaync swww swaybg wofi fuzzel tofi bemenu NetworkManager; do
        if command -v "$tool" >/dev/null 2>&1; then
            printf '  found command: %s\n' "$tool"
            found=1
        fi
    done
    for tool in waybar eww ags quickshell hyprpanel dunst swaync swww swaybg wofi fuzzel tofi bemenu; do
        if pgrep -u "${UID:-$(id -u)}" -x "$tool" >/dev/null 2>&1; then
            printf '  running process: %s\n' "$tool"
            found=1
        fi
    done
    if command -v systemctl >/dev/null 2>&1 && systemctl is-active --quiet NetworkManager.service 2>/dev/null; then
        printf '  active service: NetworkManager (Amane network menu uses systemd-networkd; no service state was changed)\n'
        found=1
    fi
    if command -v systemctl >/dev/null 2>&1 && ! systemctl is-active --quiet systemd-networkd.service 2>/dev/null; then
        printf '  note: systemd-networkd is not active; the Amane network menu may be unavailable until it is configured. The installer leaves network services unchanged.\n'
        found=1
    fi
    if [[ -f "$HYPR_HOME/hyprland.conf" ]] && grep -En '^[[:space:]]*(exec(-once)?|bind[a-z]*)[[:space:]]*=.*(waybar|eww|ags|quickshell|hyprpanel|dunst|swaync|swww|swaybg|wofi|fuzzel|tofi|bemenu)' "$HYPR_HOME/hyprland.conf"; then
        found=1
    fi
    if [[ -f "$HYPR_HOME/hyprland.lua" ]] && grep -En 'hl\.(exec_cmd|bind).*(waybar|eww|ags|quickshell|hyprpanel|dunst|swaync|swww|swaybg|wofi|fuzzel|tofi|bemenu)' "$HYPR_HOME/hyprland.lua"; then
        found=1
    fi
    if (( found == 0 )); then printf '  no common conflicts detected.\n'; fi
}

if (( REPLACE_CONFLICTS )); then
    for file in "$HYPR_HOME/hyprland.conf" "$HYPR_HOME/hyprland.lua"; do
        [[ -f "$file" ]] || continue
        if grep -Eq '(waybar|eww|ags|quickshell|hyprpanel|dunst|swaync|swww|swaybg|wofi|fuzzel|tofi|bemenu)' "$file"; then
            backup_file "$file" "config/hypr/$(basename -- "$file")"
            tmp="$(mktemp)"
            comment='--'
            [[ "$file" == *.conf ]] && comment='#'
            awk -v comment="$comment" '
                /waybar|eww|ags|quickshell|hyprpanel|dunst|swaync|swww|swaybg|wofi|fuzzel|tofi|bemenu/ {
                    if ($0 ~ /^[[:space:]]*#/ || $0 ~ /^[[:space:]]*--/) { print; next }
                    if ($0 ~ /^[[:space:]]*(exec(-once)?|bind[a-z]*)[[:space:]]*=/ || $0 ~ /hl\.(exec_cmd|bind)/) {
                        print comment " disabled by Goad OS installer: " $0
                        next
                    }
                }
                { print }
            ' "$file" >"$tmp"
            install_file "$tmp" "$file" "config/hypr/$(basename -- "$file")"
            rm -f -- "$tmp"
        fi
    done
fi
scan_conflicts

if (( SYSTEM_GREETER )); then
    if [[ ! -f /etc/greetd/config.toml ]]; then
        printf 'Cannot find /etc/greetd/config.toml.\n' >&2
        exit 1
    fi
    if [[ -e /etc/greetd/config.toml ]]; then
        sudo cp -a /etc/greetd/config.toml "/etc/greetd/config.toml.goad-os-backup-$(date +%Y%m%d-%H%M%S)"
    fi
    sudo install -D -m 644 "$ROOT/greetd/config.toml" /etc/greetd/config.toml
    printf 'Installed themed greetd config. Ensure tuigreet and start-hyprland are installed before rebooting.\n'
fi

printf '\nAmane is installed at %s. Log out and back in to load the Hyprland integration.\n' "$BIN_HOME/amane-shell"
printf 'Backups (when replacements were needed): %s\n' "$BACKUP_ROOT"
printf 'Install runtime tools such as rofi-wayland, mako, hyprpaper, xorg-xrandr, PipeWire/WirePlumber, and Python 3 if they are missing.\n'
