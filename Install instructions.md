# Goad OS / Amane laptop installer

This project contains the Amane status bar, its Rofi menus, and matching
Hyprland, Mako, Fastfetch, Qt, and HyprFM configuration. The Rust UI library is
included in `vendor/amane`, so the laptop does not need this machine's cache
directory or source checkout to build it.

## Install

Create a clean transfer archive with:

```sh
./bundle.sh
```

Copy `goad-os-amane.tar.gz` to the laptop and extract it into a new folder.
Enter the extracted project directory (the folder name depends on the archive's
parent directory), then run:

```sh
./install.sh
```

The installer builds a release binary and installs it under
`~/.local/bin/amane-shell`. It installs the app scripts and theme files under
`~/.config/amane`, installs optional app themes in their XDG config locations,
installs every bundled file from `hypr/` into `~/.config/hypr` (including
`input.lua` and `deep-current.lua`), and adds `require("deep-current")` to an
existing Hyprland Lua config. Log out and back in after installation. If the
Hyprland config is not Lua, follow the message from the installer and add the
integration in the appropriate config.

Press `Super+W` to open the wallpaper and theme picker. It can apply one image
to all monitors or choose a monitor and assign it its own image from
`~/Pictures/wallpapers`. Per-monitor assignments and the all-monitor fallback
are restored at login. The picker also switches the Rofi menus between Default
Blue and Ragebait Red; the theme updates the Amane bar, Mako notifications,
and Kitty colors. Wallpaper and theme selections are saved under
`~/.local/state/amane`.

Rust/Cargo and a working Hyprland Lua setup are required to build and use the
bar. The first build downloads Rust crate dependencies, so it needs an internet
connection and the system development libraries for Fontconfig, PulseAudio,
Wayland, and Vulkan. Menus and services use Rofi (preferably `rofi-wayland`), Mako, Hyprpaper,
Hyprland utilities, PipeWire/WirePlumber (`wpctl` and `pw-dump`), Python 3,
`flock`, and standard shell utilities. Optional themes are for Fastfetch,
Qt5CT/Qt6CT, and HyprFM. The main-monitor picker also uses `xrandr` from
`xorg-xrandr`.

The `MON` button beside the power button selects a monitor from Hyprland and
marks its matching XRandR output as primary for X11/XWayland apps. It also
moves workspace 1 there, reloads the saved workspace rule, and restores that
monitor as the default at the next Hyprland login.

On CachyOS/Arch, the main runtime packages are commonly available as `rofi`
(or `rofi-wayland` where packaged), `mako`, `hyprpaper`, `pipewire`,
`wireplumber`, `python`, and `util-linux`; install the matching Hyprland and
Rust toolchain packages for the laptop. Package names can vary by enabled
repositories.

## Existing tools and backups

The installer reports common competing bars, launchers, notification daemons,
wallpaper daemons, and NetworkManager. It leaves installed packages and system
services alone. Pass `--replace-conflicts` to comment out recognized competing
startup lines and Wofi/Tofi/Fuzzel/Bemenu bindings in `hyprland.conf` or
`hyprland.lua`; each edited file is backed up first. Running processes are
reported but not killed.

Every replaced user config file is backed up under
`~/.local/state/goad-os-backups/<timestamp>/`. The optional
`--system-greeter` flag backs up and replaces `/etc/greetd/config.toml` with
a bundled tuigreet theme and requires `sudo`. This repository currently does
not include `greetd/config.toml`, so the option exits safely without changing
system files. Do not use it until that configuration has been added and tested.

```sh
./install.sh --replace-conflicts
```
