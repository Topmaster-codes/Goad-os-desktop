------------------
---- MONITORS ----
------------------

-- See https://wiki.hypr.land/configuring/core/monitors/
hl.monitor({
    mode = "perferred",
    scale = "auto",
    position = "auto",
})

hl.config({
    render = {
        direct_scanout = 0,    -- 0 completely disables direct scanout
        cm_auto_hdr = 0,       -- 0 prevents automatic HDR/color profile shifting in fullscreen
    }
})