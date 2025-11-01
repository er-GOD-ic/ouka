local base = require('keycode.base')
local alias = require('keycode.alias')

local keychron = ouka.getDeviceByName("keychron")
keychron.keycode = ouka.margeTable(base, alias)

--[[
keychron.map("abc", function()
    print("hello world")
end)
]]

