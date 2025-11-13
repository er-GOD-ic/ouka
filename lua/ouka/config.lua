local base = require('keycode.base')
local alias = require('keycode.alias')

local value = require('keyvalue.base')

local kbd = ouka.getDeviceById("/dev/input/by-id/kmonad-main")
kbd:setKeycodes(base, alias)
kbd:setKeyValues(value)

kbd:map("c-a", function()
    print("hello world")
end)

kbd:listen();

