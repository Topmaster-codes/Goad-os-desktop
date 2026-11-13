------------------
---- MONITORS ----
------------------

-- See https://wiki.hypr.land/configuring/core/monitors/

hl.monitor({
    output = "auto",
    mode = "highrr",
    scale = "1",
    position = "auto",
})

hl.config({
    render = {
        direct_scanout = 0, -- 0 completely disables direct scanout
        cm_auto_hdr = 0,    -- 0 prevents automatic HDR/color profile shifting in fullscreen
    }
})
