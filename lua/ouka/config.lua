local base = require('keycode.base')
local alias = require('keycode.alias')

local value = require('keyvalue.base')

local kbd = ouka.getDeviceById("/dev/input/by-id/kmonad-main")
kbd:setKeycodes(base, alias)
kbd:setKeyValues(value)

kbd:map("^leftalt", function()
    os.execute("fcitx5-remote -c")
end)

kbd:map("^rightalt", function()
    os.execute("fcitx5-remote -o")
end)

kbd:map("b", function()
    kbd:ungrab()
end)

kbd:map("s-c-a-delete", function()
    ouka.kill()
end)

kbd:grab();
kbd:listen();
