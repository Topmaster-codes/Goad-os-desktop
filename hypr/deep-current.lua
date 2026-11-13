-- Deep Current: Amane integration for Hyprland 0.55+ Lua configuration.
-- This is loaded from ~/.config/hypr/hyprland.lua with require("deep-current").

-- The monitor picker saves its choice here. Reading it during config load makes
-- workspace 1's default monitor persistent without adding duplicate live rules.
local state_home = os.getenv("XDG_STATE_HOME")
if not state_home or state_home == "" then
    state_home = (os.getenv("HOME") or "") .. "/.local/state"
end
local monitor_state = io.open(state_home .. "/amane/main-monitor", "r")
if monitor_state then
    local main_monitor = monitor_state:read("*l")
    monitor_state:close()
    if main_monitor and main_monitor:match("^[%w_.:-]+$") then
        hl.workspace_rule({ workspace = "1", monitor = main_monitor, default = true })
    end
end


-- Use the Qt 6 configuration tool for Qt applications.
hl.env("QT_QPA_PLATFORMTHEME", "qt6ct")

hl.layer_rule({
    name = "deep-current-amane-bar-blur",
    match = { namespace = "^amane-bar$" },
    blur = true,
    blur_popups = true,
    ignore_alpha = 0.12,
    xray = false,
})

-- Blur translucent Rofi menus and Mako notification surfaces.
hl.layer_rule({
    name = "deep-current-rofi-blur",
    match = { namespace = "^rofi$" },
    blur = true,
    blur_popups = true,
    ignore_alpha = 0.05,
    xray = false,
})
hl.layer_rule({
    name = "deep-current-mako-blur",
    match = { namespace = "^mako$" },
    blur = true,
    blur_popups = true,
    ignore_alpha = 0.05,
    xray = false,
})

hl.curve("deepCurrentBarSpring", {
    type = "spring",
    mass = 1.0,
    stiffness = 220.0,
    dampening = 13.0,
})
hl.animation({ leaf = "layers", enabled = true, speed = 1.0, spring = "deepCurrentBarSpring", style = "slide top" })
hl.animation({ leaf = "layersIn", enabled = true, speed = 1.0, spring = "deepCurrentBarSpring", style = "slide top" })
hl.animation({ leaf = "layersOut", enabled = true, speed = 1.0, spring = "deepCurrentBarSpring", style = "slide top" })

hl.curve("deepCurrentWorkspaceSpring", {
    type = "spring",
    mass = 1.0,
    stiffness = 90.0,
    dampening = 14.0,
})
hl.animation({ leaf = "workspaces", enabled = true, speed = 7.0, spring = "deepCurrentWorkspaceSpring", style = "slidefade 24%" })
hl.animation({ leaf = "workspacesIn", enabled = true, speed = 7.0, spring = "deepCurrentWorkspaceSpring", style = "slidefade 24%" })
hl.animation({ leaf = "workspacesOut", enabled = true, speed = 7.0, spring = "deepCurrentWorkspaceSpring", style = "slidefade 24%" })

hl.unbind("SUPER + SPACE")
hl.bind("SUPER + SPACE", hl.dsp.exec_cmd([[sh -lc '"${XDG_CONFIG_HOME:-$HOME/.config}/amane/scripts/rofi-single" -show drun']]))
hl.bind("SUPER + O", hl.dsp.exec_cmd([[sh -lc '"${XDG_CONFIG_HOME:-$HOME/.config}/amane/scripts/power-profile-menu"']]))
hl.bind("SUPER + W", hl.dsp.exec_cmd([[sh -lc '"${XDG_CONFIG_HOME:-$HOME/.config}/amane/scripts/wallpaper-menu"']]))

-- Start Mako and Hyprpaper when those packages are installed.
hl.on("hyprland.start", function()
    hl.exec_cmd([[sh -lc 'CFG="${XDG_CONFIG_HOME:-$HOME/.config}"; "$CFG/amane/scripts/start-amane-services"']])
end)
