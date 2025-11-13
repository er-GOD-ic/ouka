local base = require('keycode.base')
local alias = require('keycode.alias')

local kbd = ouka.getDeviceById("/dev/input/by-id/kmonad-main")
kbd:setKeycodes(base, alias)

kbd:map("a", function()
    print("hello world")
end)

kbd:listen();

