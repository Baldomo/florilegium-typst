-- Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
-- Shared by the oracle scripts: finds the Lua original (env FLORILEGIUM_LUA, default
-- <lib>/../florilegium), stubs TeX out and captures what node.write is given.
local M = {}

local here = (arg and arg[0] or ""):match("^(.*)/[^/]*$") or "."
M.dir = os.getenv("FLORILEGIUM_LUA") or (here .. "/../../../florilegium")
package.path = M.dir .. "/?.lua;" .. here .. "/?.lua;" .. package.path

tex = { sprint = function() end }
luatexbase = { registernumber = function() return 0 end }
local streams = {}
node = { new = function() return {} end, write = function(n) streams[#streams + 1] = n.data end }

function M.reset() streams = {} end

-- the streams written since the last reset, joined as the Rust side joins them
function M.streams() return table.concat(streams, "\n=====\n") end

return M
