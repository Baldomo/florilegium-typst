-- Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
-- The case registry. A family registers `ids()` (its case ids) and `run(id)` (the output
-- of one case as a string); the id's first path segment selects the family.
local common = require("common")
local M = {}
local families = {}
local order = {}

function M.register(name, ids, run)
  families[name] = { ids = ids, run = run }
  order[#order + 1] = name
end

function M.ids()
  local all = {}
  for _, name in ipairs(order) do
    for _, id in ipairs(families[name].ids()) do all[#all + 1] = id end
  end
  return all
end

function M.run(id)
  local fam = families[id:match("^([^/]+)/")]
  if not fam then error("unknown case family in id: " .. id) end
  common.reset()
  return fam.run(id)
end

-- ---------------------------------------------------------------- fmt
-- mulberry32 (Tommy Ettinger, 2017, public domain), the same as ink.rng
local function rng(seed)
  local s = math.floor(seed) & 0xffffffff
  return function()
    s = (s + 0x6D2B79F5) & 0xffffffff
    local t = s
    t = ((t ~ (t >> 15)) * (t | 1)) & 0xffffffff
    t = (t ~ (t + (((t ~ (t >> 7)) * (t | 61)) & 0xffffffff))) & 0xffffffff
    return ((t ~ (t >> 14)) & 0xffffffff) / 4294967296
  end
end

local FMT = { f1 = "%.1f", f2 = "%.2f", f3 = "%.3f", g = "%g", g14 = "%.14g" }
local FMT_SPECS = { "f1", "f2", "f3", "g", "g14" }

M.register("fmt", function()
  local ids = {}
  for _, spec in ipairs(FMT_SPECS) do
    for k = 1, 4 do ids[#ids + 1] = ("fmt/%s/seed=%d"):format(spec, k) end
  end
  return ids
end, function(id)
  local spec, seed = id:match("^fmt/([^/]+)/seed=(%d+)$")
  local f = FMT[spec]
  if not f then error("bad fmt id: " .. id) end
  local u = rng(tonumber(seed))
  local out = {}
  for _ = 1, 20000 do
    local a = u()
    local e = math.floor(u() * 12) - 4
    out[#out + 1] = f:format((a - 0.5) * 10 ^ e)
  end
  for n = -400, 400 do out[#out + 1] = f:format(n / 20) end
  for n = -400, 400 do out[#out + 1] = f:format(n / 200) end
  return table.concat(out, "\n") .. "\n"
end)

return M
