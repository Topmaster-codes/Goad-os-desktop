# CLAUDE.md — Goad-OS desktop (Amane / Hyprland / Rofi)

Pre-analysis of this repo so you don't have to re-explore it. Trust it, but check
anything you are about to depend on: it was written from a read of the code, and
code changes.

## What this is

Config and installer for a CachyOS/Arch desktop: a Rust status bar (`amane-shell`,
built on the vendored `amane` UI library), Rofi menus, Hyprland integration and app
themes. It is **not** a bootable distro yet, and the local AI assistant in the README
is a future goal with no code. GPL-3.0-or-later; `vendor/amane` is MIT.

## Read this, not that

- Real docs: `Install instructions.md`. `README.md` is mostly marketing, and its
  packaging text is wrong (it says `.7z` and `./config/amane`; `bundle.sh` makes a
  `.tar.gz`).
- Don't read or broadly grep `vendor/amane` (184 files, ~20k lines of upstream
  library), `Cargo.lock` or `LICENSE`. The whole shell is `src/main.rs`; if you must
  look in the library, grep for one symbol.
- `src/main.rs` is 1,126 lines: grep for the symbol you need instead of reading it.

## Layout

- `src/main.rs`: the entire bar. `Palette` (blue/red), `Service` structs (clock,
  CPU/RAM, networkd, notification history, power daemon), chips and panels, `bar()`,
  `main()`. Buttons run scripts through `launch_amane_script(name)`: audio-device-menu,
  network-manager-menu, power-profile-menu, power-menu, monitor-manager-menu.
- `scripts/` (installed to `~/.config/amane/scripts`; bash, except the Python
  `audio-applications`):
  - `rofi-single`: every menu goes through it. Picks `rofi-wayland`, else `rofi`;
    flock single-instance lock; chooses `rofi/amane.rasi` or `amane-red.rasi` from the
    saved theme.
  - `wallpaper-menu`: Super+W. Wallpaper picker (images via Hyprpaper, videos via
    mpvpaper) plus the theme picker.
  - `wallpaper-lib`: sourced helpers: `read_state`, `wallpaper_is_video`, `mpvpaper_*`.
  - `apply-theme`: applies the saved theme to Mako and Kitty, restarts `amane-shell`.
  - `start-amane-services`: run on `hyprland.start`. Applies theme, restores main
    monitor, starts the bar and Hyprpaper, restores saved wallpapers.
  - `monitor-manager-menu` (main monitor via xrandr + workspace 1; `--apply-saved` at
    login), `audio-device-menu` + `audio-applications` (PipeWire),
    `network-manager-menu` (systemd-networkd), `power-menu`, `power-profile-menu`.
- `hypr/`: `hyprland.lua` requires `input`, `looks`, `monitors`, `deep-current`.
  `deep-current.lua` is the Amane integration: workspace-1 rule from the saved main
  monitor, layer blur and animation rules, keybinds (Super+Space drun, Super+O power
  profile, Super+W wallpaper), starts `start-amane-services`. Uses the Hyprland 0.55+
  Lua API (`hl.*`).
- Themes: `rofi/amane*.rasi`, `mako*.conf`, `kitty/themes/*`, each in a blue and a red
  variant. `hyprfm/`, `qt5ct/`, `qt6ct/`, `fastfetch/` and `hypr/hyprlock.conf` are
  teal/static and do not follow the theme switch.
- `install.sh` (builds, then installs `scripts/ rofi/ kitty/` to `~/.config/amane`
  and `hypr/` to `~/.config/hypr`), `bundle.sh`, `.github/workflows/ci.yml`.

## State and runtime files

State, in `${XDG_STATE_HOME:-~/.local/state}/amane/`, one line each:
`theme` (`default-blue` or `ragebait-red`; anything else means blue), `wallpaper`
(global path), `wallpapers-by-monitor/<monitor>` (per-monitor path, overrides the
global one), `main-monitor`. A wallpaper state file holds a path and the extension
decides image vs video.

Runtime, in `RUNTIME_DIR=${XDG_RUNTIME_DIR:-${XDG_CACHE_HOME:-~/.cache}/amane}`. With
`XDG_RUNTIME_DIR` set that is the bare runtime dir, so files land directly in it:
`mako.conf`, `*.log`, `amane-mpvpaper/<monitor>.pid`. The rofi lock is
`$XDG_RUNTIME_DIR/deep-current/rofi.lock`.

## Themes are hardcoded in four code files

Ids are `default-blue` and `ragebait-red`. A new theme must touch `palette()` in
`src/main.rs`, `scripts/rofi-single`, both `case` blocks in `scripts/apply-theme`, and
the theme labels and `case` arms in `scripts/wallpaper-menu`, plus the paired
rofi/mako/kitty files. The bar reads its palette once at startup, which is why
`apply-theme` restarts it.

## Script conventions

All scripts: `#!/usr/bin/env bash`, an `# SPDX-License-Identifier: GPL-3.0-or-later`
header and `set -euo pipefail`. The menu scripts (everything launched from the bar or a
keybind) also use `SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"`
and a local `notify()` that falls back to stderr, and call
`rofi-single -dmenu -format s -p ... -l N`. Validate monitor names against
`^[A-Za-z0-9_.:-]+$`. Write state atomically (temp file, then `mv -f`). Background
daemons: `cmd >>log 2>&1 &` then `disown`.

## Gotchas

1. **Never read state with `$(<f 2>/dev/null || true)`.** It always yields an empty
   string (bash only reads the file for a bare `$(<f)`). It silently broke theme
   apply, wallpaper restore and main-monitor restore until fixed. Use `read_state`
   from `wallpaper-lib`, or `IFS= read -r v <"$f"` behind a `[[ -r "$f" ]]` check.
2. **`bundle.sh` archives `git ls-files`**, so `git add` new files. `install.sh` copies
   every file under `scripts/`, `rofi/` and `kitty/`; the executable bit decides mode
   755 vs 644, so sourced libraries stay non-executable and new executables need
   `chmod +x` before commit.
3. **CI doesn't check most scripts.** It runs Rust fmt, check, clippy `-D warnings` and
   a release build, plus `bash -n install.sh bundle.sh` and `py_compile
   scripts/audio-applications` only. Run `bash -n` on any script you touch.
4. **Rofi:** the user runs plain `rofi` 2.0.0. Rofi 2.1.1 broke the squash/stretch
   resize animation when the number of listed items changes.
5. **Video wallpapers:** one mpvpaper process per monitor, tracked by PID file.
   Overrides: `AMANE_MPVPAPER_OPTIONS`, `AMANE_MPVPAPER_ARGS`. `--auto-pause` is opt-in
   because its syntax differs between mpvpaper versions.
6. Branding: don't rename "Goad-OS" or edit `BRANDING.md` or `fastfetch/goad-os.txt`
   unless asked.

## Working in the agent sandbox

- Missing there (checked): `hyprctl`, `rofi`, `mpvpaper`, `mpv`, `shellcheck`. Present:
  bash, jq, python3, cargo.
- `cargo check` fails offline (no crates.io cache, no network installs). Rust edits
  can't be compile-checked there: say so rather than claim they build, keep them
  small, and match the surrounding code (rustfmt, clippy `-D warnings`).
- **Testing scripts without Hyprland** works well. Use a temp `HOME` with
  `XDG_CONFIG_HOME`, `XDG_STATE_HOME` and `XDG_RUNTIME_DIR` inside it. Copy `scripts/`
  and `rofi/` to `$XDG_CONFIG_HOME/amane/`. Put stubs for `hyprctl` (`-j monitors`
  JSON, `hyprpaper ...`), `rofi` (reads the menu on stdin, answers from a file so
  multi-step menus are scriptable), `notify-send`, `mpvpaper`, `hyprpaper`, `mako` and
  `xrandr` first in `PATH`, then assert on the calls they log and on the state files.
  To prove a bug, run the script from `git show HEAD:path` next to the fixed one.
