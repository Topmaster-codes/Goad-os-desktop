------------------
---- MONITORS ----
------------------

-- See https://wiki.hypr.land/configuring/core/monitors/
hl.monitor({
    output   = "DP-3",
    mode     = "1920x1080@300",
    position = "0x0",
    scale    = "1",
    bitdepth = 10
})

hl.monitor({
    output = "HDMI-A-1",
    mode = "1360x768@60",
    position = "1920x0",
    scale = "1"
})

hl.config({
    render = {
        direct_scanout = 0,    -- 0 completely disables direct scanout
        cm_auto_hdr = 0,       -- 0 prevents automatic HDR/color profile shifting in fullscreen
    }
})