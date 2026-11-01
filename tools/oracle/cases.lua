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

-- ---------------------------------------------------------------- ink
local ink = require("ink")
local svg = require("svg")
local sfmt = string.format

local function G(x) return sfmt("%.17g", x) end
local function nums(out, ...)
  local t = { ... }
  for i = 1, select("#", ...) do t[i] = G(t[i]) end
  out[#out + 1] = table.concat(t, " ")
end
local function flat(out, label, pts)
  local t = { label }
  for _, p in ipairs(pts) do t[#t + 1] = G(p[1]) t[#t + 1] = G(p[2]) end
  out[#out + 1] = table.concat(t, " ")
end
local function finish(out) return table.concat(out, "\n") .. "\n=====\n" .. common.streams() end

-- an HTML color as svg.lua's html() hands it to ink (xcolor's n/255 to five decimals)
local function print_scaled(s)
  local o, delta = { tostring(s // 65536), "." }, 10
  s = 10 * (s % 65536) + 5
  repeat
    if delta > 65536 then s = s + 0x8000 - 50000 end
    o[#o + 1] = tostring(s // 65536)
    s, delta = 10 * (s % 65536), delta * 10
  until s <= delta
  return table.concat(o)
end
local function xcolor(n)
  local q, r, digits = n // 255, n % 255, {}
  for k = 1, 6 do r = r * 10 digits[k] = r // 255 r = r % 255 end
  local s = svg.sp(q .. "." .. table.concat(digits, "", 1, 5) .. "pt")
  if digits[6] > 4 then s = s + 1 end
  return tonumber((print_scaled(s):gsub("%.0$", "")))
end
local function html(h)
  return xcolor(tonumber(h:sub(1, 2), 16)), xcolor(tonumber(h:sub(3, 4), 16)), xcolor(tonumber(h:sub(5, 6), 16))
end
M.html = html

local function ramp_lines(out)
  for i = 0, 100 do local c = ink.tone(i / 100) nums(out, c[1], c[2], c[3]) end
  for i = -10, 10 do local c = ink.accent(i / 10) nums(out, c[1], c[2], c[3]) end
  ink.in_accent(function() for i = 0, 100 do local c = ink.tone(i / 100) nums(out, c[1], c[2], c[3]) end end)
end

local INKS = { { "2456C4", "B4662F" }, { "5B2A1F", "2F6B3A" }, { "808080", "000000" } }
local PALES = { "0", "0.3", "0.85" }
local RNG_SEEDS = { "1", "7", "4294967295", "123456789.7" }
local NSEEDS = { "1", "7", "4294967295", "123456789.7" }
local DRAWS = { "paths", "transforms", "geometry", "recording", "colors" }

M.register("ink", function()
  local ids = {}
  for _, s in ipairs(RNG_SEEDS) do ids[#ids + 1] = "ink/rng/seed=" .. s end
  for _, s in ipairs(NSEEDS) do ids[#ids + 1] = "ink/noise/seed=" .. s end
  for _, p in ipairs(INKS) do ids[#ids + 1] = ("ink/ramp/ink=%s/accent=%s"):format(p[1], p[2]) end
  for _, v in ipairs(PALES) do ids[#ids + 1] = "ink/pale/v=" .. v end
  for _, d in ipairs(DRAWS) do ids[#ids + 1] = "ink/draw/case=" .. d end
  return ids
end, function(id)
  local kind, rest = id:match("^ink/([^/]+)/(.*)$")
  local out = {}
  if kind == "rng" then
    local r = ink.rng(tonumber(rest:match("^seed=(.*)$")))
    for _ = 1, 2000 do nums(out, r.u()) end
    for _ = 1, 2000 do nums(out, r.range(-3, 5)) end
    for _ = 1, 2000 do nums(out, r.int(1, 6)) end
    for _ = 1, 2000 do nums(out, r.normal()) end
    for _ = 1, 500 do nums(out, r.sym(2.5)) end
    for _ = 1, 500 do nums(out, r.chance(0.3) and 1 or 0) end
    for _ = 1, 500 do nums(out, r.pick({ 10, 20, 30, 40, 50 })) end
    local s = r.save()
    nums(out, s, r.u(), r.u())
    r.load(s)
    nums(out, r.u())
    for _ = 1, 200 do r = r.fork() nums(out, r.u()) end
    return table.concat(out, "\n") .. "\n"
  elseif kind == "noise" then
    local r = ink.rng(tonumber(rest:match("^seed=(.*)$")))
    local f = ink.fbm1(r, 3)
    for i = 0, 499 do nums(out, f(i * 0.37 - 20)) end
    local n = ink.noise1(r)
    for i = 0, 99 do nums(out, n(i * 0.91 - 40)) end
    return table.concat(out, "\n") .. "\n"
  elseif kind == "ramp" or kind == "pale" then
    if kind == "ramp" then
      local i, a = rest:match("^ink=(%x+)/accent=(%x+)$")
      ink.set_ink(html(i))
      ink.set_accent(html(a))
    else
      ink.set_pale(tonumber(rest:match("^v=(.*)$")))
    end
    ramp_lines(out)
    return table.concat(out, "\n") .. "\n"
  end
  local name = rest:match("^case=(.*)$")
  local tone, accent = ink.tone, ink.accent
  local PTS = { { 0, 0 }, { 1, 0.5 }, { 2, 0.2 }, { 3, 1 }, { 3.5, 2 } }
  if name == "paths" then
    ink.layer(3)
    ink.moveto(1, 1) ink.lineto(3, 1) ink.lineto(3, 3) ink.close() ink.fill(tone(0.3))
    ink.moveto(0, 0) ink.curveto(1, 2, 2, 2, 3, 0) ink.stroke(tone(0.1), 0.5)
    ink.poly({ { 0, 0 }, { 1, 0.5 }, { 2, 0 } }, false) ink.stroke(tone(0.2), 0.25, 0.5, false, "[1.2 1.6] 0")
    ink.poly({ { 0, 0 }, { 1, 0.5 }, { 2, 0 } }, true) ink.fill(accent(0.3), 0.4, true)
    ink.smooth({ { 0, 0 }, { 1, 1 } }, true) ink.stroke(tone(0.4), 0.3)
    ink.smooth({ { 0, 0 }, { 1, 1 }, { 2, 0 } }, false) ink.stroke(tone(0.4), 0.3, 1)
    ink.smooth(PTS, false) ink.fill(tone(0.5), 0.85)
    ink.smooth(PTS, true, 0.7) ink.stroke(tone(0.35), 0.8, 0.999, true)
    ink.smooth(PTS, false, nil) ink.stroke(accent(-0.4), 1.25, 0.304)
    ink.circle(2, 2, 0.75) ink.stroke(ink.white, 1, 1, true)
    ink.circle(2, 2, 0.75) ink.clip()
    ink.flood(tone(0.6), -1, -1, 5, 5)
    ink.circle(2.5, 2, 0.4) ink.fill(ink.paper, 0.5)
    ink.unclip()
    ink.moveto(9, 9) ink.lineto(1, 1) ink.discard()
    ink.layer(7) ink.circle(1, 1, 1) ink.fill(tone(0.9), 0.123456)
    ink.layer(1) ink.poly({ { 0, 0 }, { 1, 0 }, { 1, 1 } }, true) ink.fill(tone(0.05))
    ink.layer(5) ink.poly({ { 0, 0 }, { -1, 0 }, { -1, -1 } }, true) ink.fill(tone(0.15), 1, false)
    nums(out, ink.layer(2), ink.layer(5))
    nums(out, ink.flush())
    nums(out, ink.flush())
  elseif name == "transforms" then
    nums(out, table.unpack(ink.get_transform()))
    nums(out, table.unpack(ink.page_transform()))
    nums(out, table.unpack(ink.page_transform(21)))
    ink.push()
    ink.translate(1, 2) ink.rotate(30) ink.scale(2)
    nums(out, table.unpack(ink.get_transform()))
    nums(out, ink.unit())
    nums(out, ink.to_page(0.5, 0.25))
    ink.poly({ { 0, 0 }, { 1, 0 }, { 0, 1 } }, true) ink.fill(tone(0.3))
    ink.push()
    ink.scale(1, 3) ink.rotate(-75.5) ink.translate(-0.3, 0.2)
    nums(out, table.unpack(ink.get_transform()))
    nums(out, ink.unit())
    nums(out, ink.to_page(-1, 2))
    ink.circle(0, 0, 0.5) ink.stroke(tone(0.2), 0.3)
    ink.pop()
    nums(out, table.unpack(ink.get_transform()))
    ink.pop()
    nums(out, table.unpack(ink.get_transform()))
    ink.set_transform(ink.page_transform(21))
    ink.poly({ { 1, 1 }, { 2, 1 }, { 2, 2 } }, true) ink.fill(tone(0.25))
    nums(out, ink.to_page(3, 4))
    ink.set_transform(ink.page_transform(nil))
    ink.poly({ { 1, 1 }, { 2, 1 }, { 2, 2 } }, true) ink.fill(tone(0.25))
    ink.flush()
  elseif name == "geometry" then
    local DEG = { { 0.5, 0.5 }, { 0.5, 0.5 }, { 1.5, 1.5 }, { 1.5, 1.5 } }
    local nr = ink.normals(PTS) flat(out, "normals", nr)
    flat(out, "normals-degenerate", ink.normals(DEG))
    local s, L = ink.arclen(PTS)
    nums(out, L, table.unpack(s))
    s, L = ink.arclen({ { 1, 1 } })
    nums(out, L, table.unpack(s))
    local rb = ink.ribbon(PTS, function(u) return 0.2 + 0.3 * u end) flat(out, "ribbon", rb)
    ink.poly(rb, true) ink.fill(tone(0.3))
    flat(out, "ribbon-point", ink.ribbon({ { 1, 1 } }, function(u) return 0.5 + u end))
    ink.taper(0, 0, 1, 2, 2, 2, 3, 0, 0.4, 0.05) ink.fill(tone(0.2))
    ink.taper(0, 0, 0, 0, 2, 2, 2, 2, 0.4, 0.05) ink.fill(tone(0.2))
    flat(out, "catmull", ink.catmull(PTS, 4))
    flat(out, "catmull2", ink.catmull({ { 0, 0 }, { 2, 3 } }, 3))
    flat(out, "catmull1", ink.catmull({ { 5, 7 } }, 3))
    local b3 = ink.bezier3(0, 0, 1, 2, 2, 2, 3, 0)
    nums(out, b3(0)) nums(out, b3(0.3)) nums(out, b3(1))
    flat(out, "sample", ink.sample(b3, 8))
    flat(out, "sample-range", ink.sample(b3, 5, 0.2, 0.8))
    local spine = function(v) return v * 2, math.sin(v * 3) * 0.5 end
    local wl = function(v) return 0.3 * math.sin(v * math.pi) end
    local wr = function(v) return 0.25 * math.sin(v * math.pi) + 0.05 end
    local bl, c, nrm = ink.blade(spine, 6, wl, wr)
    flat(out, "blade", bl) flat(out, "blade-c", c) flat(out, "blade-n", nrm)
    ink.smooth(bl, true) ink.fill(tone(0.4))
    bl, c, nrm = ink.blade(spine, 7, wl, wr, function(v, side) return side * 0.1 * math.cos(v * 9) - 0.2 end)
    flat(out, "blade-edge", bl) flat(out, "blade-edge-c", c) flat(out, "blade-edge-n", nrm)
    local r = ink.rng(5)
    flat(out, "blob", ink.blob(r, 1, 2, 1.5, 1, 0.3))
    flat(out, "blob5", ink.blob(r, -1, 0.5, 0.5, 1.5, 0.2, 5))
    nums(out, r.u())
    ink.flush()
  elseif name == "recording" then
    ink.poly({ { 0, 0 }, { 1, 0 }, { 1, 1 } }, true) ink.fill(tone(0.1))
    ink.record(true)
    local e1, e2 = { t0 = 0, dur = 1 }, { t0 = 1, dur = 2 }
    nums(out, ink.tag({ e1, 0 }) == nil and 1 or 0)
    ink.circle(1, 1, 1) ink.fill(tone(0.2))
    local old = ink.tag({ e1, 0.5, 1 })
    nums(out, old[2], old[3] == nil and 1 or 0)
    ink.circle(2, 1, 1) ink.fill(tone(0.3))
    ink.layer(2)
    ink.tag({ e2, 0, nil, overlay = true })
    ink.circle(3, 1, 1) ink.fill(tone(0.4))
    ink.tag(nil)
    ink.circle(4, 1, 1) ink.fill(tone(0.5))
    ink.layer(5)
    old = ink.tag({ e2, 1 })
    ink.circle(5, 1, 1) ink.fill(tone(0.6))
    ink.tag(old)
    ink.circle(6, 1, 1) ink.fill(tone(0.7))
    ink.tag({ e2, 0, 0.5 })
    ink.record(true)
    ink.circle(7, 1, 1) ink.fill(tone(0.8))
    ink.tag({ e1, 0, 0.25 })
    ink.circle(8, 1, 1) ink.fill(tone(0.85))
    local rec = ink.take_recording()
    nums(out, #rec.keys, rec.keys[1], rec.keys[2])
    ink.circle(9, 9, 1) ink.fill(tone(0.9))
    nums(out, ink.flush())
    nums(out, ink.emit(rec, function() return true end))
    local t = 1.5
    nums(out, ink.emit(rec, function(tag)
      if tag.overlay then return false end
      local e = tag[1]
      local p = (t - e.t0) / e.dur
      return p >= tag[2] and (tag[3] == nil or p < tag[3])
    end))
    nums(out, ink.emit(rec, function() return false end))
    ink.record(true) ink.record(false)
    ink.circle(0, 0, 1) ink.fill(tone(0.1))
    nums(out, ink.flush())
  elseif name == "colors" then
    nums(out, ink.get_lift())
    nums(out, ink.set_lift(0.3), ink.get_lift())
    for i = 0, 10 do local c = tone(i / 10) nums(out, c[1], c[2], c[3]) end
    ink.circle(1, 1, 1) ink.fill(tone(0.2))
    nums(out, ink.set_pale(0.5))
    for i = 0, 10 do local c = tone(i / 10) nums(out, c[1], c[2], c[3]) end
    local c = accent(0.4) nums(out, c[1], c[2], c[3])
    ink.set_accent(0.2, 0.5, 0.3)
    ink.in_accent(function()
      for i = 0, 10 do local d = tone(i / 10) nums(out, d[1], d[2], d[3]) end
      ink.circle(2, 1, 1) ink.fill(tone(0.4))
      ink.in_accent(function() ink.circle(3, 1, 1) ink.fill(tone(0.5)) end)
      ink.circle(4, 1, 1) ink.fill(tone(0.6))
    end)
    ink.circle(5, 1, 1) ink.fill(tone(0.6))
    ink.set_accent(0.9, 0.2, 0.1)
    ink.in_accent(function() ink.circle(6, 1, 1) ink.fill(tone(0.6)) end)
    ink.in_accent(function() ink.circle(7, 1, 1) ink.fill(tone(0.7)) end)
    ink.set_ink(html("5B2A1F"))
    ink.in_accent(function() ink.set_ink(0.1, 0.6, 0.2) ink.circle(8, 1, 1) ink.fill(tone(0.7)) end)
    ink.circle(9, 1, 1) ink.fill(tone(0.7))
    ink.in_accent(function() ink.circle(10, 1, 1) ink.fill(tone(0.7)) end)
    ink.flush()
  else
    error("bad ink id: " .. id)
  end
  return finish(out)
end)

-- ---------------------------------------------------------------- flora
local flora = require("flora")

-- name, size key, sizes (bytes.lua, a small ORNAMENT size, a large one), then option values
local FLORA = {
  { "flower", "s", { 1.2, 0.16, 2.4 }, { petals = { 4, 6 }, tone = { 0.2, 0.5 } } },
  { "lily", "s", { 1.1, 0.21, 2.2 }, { tilt = { 0.3, 0.7 }, tone = { 0.25, 0.45 } } },
  { "blossom", "s", { 0.3, 0.17, 0.8 }, { tone = { 0.2, 0.5 }, eye = { "0.9,0.2,0.1", "0.05,0.05,0.05" } } },
  { "floret", "s", { 0.1, 0.05, 0.5 }, { star = { "true", "false" }, tone = { 0.3, 0.4 }, eye = { "0.9,0.2,0.1" } } },
  { "bud", "s", { 0.6, 0.33, 1.2 }, { tone = { 0.25, 0.5 } } },
  { "bell", "s", { 1, 0.33, 2 }, { tone = { 0.25, 0.5 } } },
  { "leaf", "scale", { 2, 0.35, 4 }, { depth = { 0, 0.6 }, w = { 0.15, 0.3 }, lobes = { 2, 6 }, bend = { 0.3, -0.3 }, tone = { 0.2, 0.45 } } },
  { "leaflet", "scale", { 0.6, 0.35, 1.5 }, { w = { 0.1, 0.3 }, bend = { 0.3, -0.3 }, tone = { 0.15, 0.4 } } },
  { "petal", "scale", { 1.2, 0.35, 3 }, { shape = { "pointed", "round" }, w = { 0.2, 0.5 }, bend = { 0.1, -0.2 }, tone = { 0.25, 0.5 },
    streaks = { 4, 11 }, heart = { "false", "true" } } },
  { "cyme", "s", { 2, 0.33, 3 }, { side = { 1, -1 }, fork = { "false", "true" } } },
  { "sprig", "s", { 2, 0.35, 3 }, { depth = { 2, 5 } } },
  { "umbel", "s", { 1.2, 0.28, 2.4 }, {} },
  { "seedhead", "s", { 1, 0.15, 2 }, { side = { 1, -1 }, drift = { 0, 5 } } },
  { "tendril", "s", { 1, 0.16, 2 }, {} },
  { "sparkle", "s", { 0.3, 0.1, 0.6 }, { rays = { "false", "true" }, color = { "0.2,0.5,0.3", "0.9,0.1,0.1" } } },
  { "rosette", "s", { 0.3, 0.13, 0.6 }, { color = { "0.2,0.5,0.3", "0.9,0.1,0.1" } } },
  { "hedera", "s", { 0.36, 0.13, 0.8 }, { color = { "0.2,0.5,0.3", "0.9,0.1,0.1" } } },
  { "butterfly", "s", { 2, 0.34, 4 }, { open = { 0.3, 0.6, 1 }, tone = { 0.2, 0.45 } } },
  { "bee", "s", { 1, 0.28, 2 }, { trail = { "false", "true" } } },
  { "snail", "s", { 1, 0.19, 2 }, { tone = { 0.4, 0.7 } } },
  { "bird", "s", { 1, 0.33, 2 }, { facing = { -1, 1 }, tone = { 0.25, 0.45 } } },
}
local GHOSTED = { "flower", "lily", "blossom", "floret", "bud", "bell", "leaf", "leaflet", "petal", "cyme", "sprig", "umbel",
  "seedhead", "butterfly", "bee", "snail", "bird" }
local GHOST_SIZES = { flower = 1.2, lily = 1.1, blossom = 0.3, floret = 0.1, bud = 0.6, bell = 1, leaf = 2, leaflet = 0.6, petal = 1.2,
  cyme = 2, sprig = 2, umbel = 1.2, seedhead = 1, butterfly = 2, bee = 1, snail = 1, bird = 1 }
local LIFTS = { "0.4", "default" }
local SPEC = {}
for _, r in ipairs(FLORA) do SPEC[r[1]] = r end
local STEMS = {
  "path=bytes/w0=0.08/w1=0.03", "path=bytes/w0=0.08/w1=0.03/tip=0.4", "path=bytes/w0=0.01/w1=0.005", "path=int/w0=0.1/w1=0.02",
  "path=int/w0=0.1/w1=0.02/tip=1", "path=int/w0=0.1/w1=0.02/c=0.9,0.2,0.1/op=0.5", "path=bytes/w0=0.01/w1=0.005/c=0.9,0.2,0.1/op=0.5",
  "path=int/w0=0.1/w1=0.02/scale=0.2", "path=bytes/w0=0.2/w1=0.2/tip=100",
}

local function sorted_keys(t) local k = {} for key in pairs(t) do k[#k + 1] = key end table.sort(k) return k end

M.register("flora", function()
  local ids = {}
  local function add(id) ids[#ids + 1] = id end
  for _, r in ipairs(FLORA) do
    for _, s in ipairs(r[3]) do
      for k = 1, 5 do add(("flora/%s/%s=%s/seed=%d"):format(r[1], r[2], tostring(s), k)) end
    end
    for _, key in ipairs(sorted_keys(r[4])) do
      for _, v in ipairs(r[4][key]) do
        for k = 1, 5 do add(("flora/%s/%s=%s/seed=%d/%s=%s"):format(r[1], r[2], tostring(r[3][1]), k, key, tostring(v))) end
      end
    end
  end
  for _, lift in ipairs(LIFTS) do
    for _, name in ipairs(GHOSTED) do
      for k = 1, 3 do add(("flora/ghost/lift=%s/%s/s=%s/seed=%d"):format(lift, name, tostring(GHOST_SIZES[name]), k)) end
    end
  end
  for _, st in ipairs(STEMS) do add("flora/stem/" .. st) end
  for k = 1, 5 do
    add("flora/mottle/specks=0/seed=" .. k)
    add("flora/mottle/specks=8/seed=" .. k)
    add("flora/branch/case=ahead/seed=" .. k)
    add("flora/branch/case=behind/seed=" .. k)
  end
  for _, c in ipairs({ "a", "b" }) do
    add("flora/streak/case=" .. c)
    add("flora/streak3/case=" .. c)
  end
  add("flora/walk/case=a")
  add("flora/walk/case=b")
  return ids
end, function(id)
  local name, rest = id:match("^flora/([^/]+)/(.*)$")
  local out = { id }
  local kv, order = {}, {}
  for key, v in rest:gmatch("([^/=]+)=([^/]*)") do kv[key] = v order[#order + 1] = key end
  local function val(v)
    if v == "true" then return true elseif v == "false" then return false end
    if tonumber(v) then return tonumber(v) end
    local a, b, c = v:match("^([^,]+),([^,]+),([^,]+)$")
    if a and tonumber(a) then return { tonumber(a), tonumber(b), tonumber(c) } end
    return v
  end
  ink.set_transform({ ink.CM, 0, 0, ink.CM, 0, 0 })
  local tone, white = ink.tone, ink.white
  if name == "stem" then
    local pts = {}
    if kv.path == "bytes" then for i = 0, 80 do pts[#pts + 1] = { math.sin(i / 9), i / 4 } end
    else pts = { { 0, 0 }, { 1, 0 }, { 2, 1 }, { 3, 1 }, { 3, 2 }, { 2, 3 }, { 2, 4 }, { 1, 5 }, { 0, 5 }, { 0, 6 } } end
    if kv.scale then ink.scale(tonumber(kv.scale)) end
    local c = kv.c and val(kv.c) or nil
    flora.stem(pts, tonumber(kv.w0), tonumber(kv.w1), c, kv.op and tonumber(kv.op) or nil, kv.tip and tonumber(kv.tip) or nil)
  elseif name == "mottle" then
    local r = ink.rng(tonumber(kv.seed))
    ink.scale(1.5)
    flora.mottle(r, -1, 0, 1, 2, 12, 0.2, 0.2, 0.6, tonumber(kv.specks))
    nums(out, r.u())
  elseif name == "branch" then
    local r = ink.rng(tonumber(kv.seed))
    local pts
    if kv.case == "ahead" then pts = flora.branch_path(r, 0, 0, 0, 1, 1, 3, 0, 1)
    else pts = flora.branch_path(r, 1, 2, 0, 1, 0.5, -1, 1, 0) end
    flat(out, "branch", pts)
    ink.smooth(pts, false) ink.stroke(tone(0.1), 0.5)
    nums(out, r.u())
  elseif name == "streak" or name == "streak3" then
    if name == "streak" then
      local f = function(v) return math.sin(v * 3), v * 4 end
      flora.streak(f, 12, 0.3, 0.02, white, kv.case == "a" and 0.8 or nil, kv.case == "b")
    else
      flora.streak3(0, 0, 0.5, 1, 1, 2, 0.5, 3, 0.2, 0.01, tone(0.3), kv.case == "a" and 0.7 or nil, kv.case == "b")
    end
  elseif name == "walk" then
    local pts = flora.walk(1, 2, function(s) return 90 + (kv.case == "a" and 30 or -20) * s end, 3, 12)
    flat(out, "walk", pts)
  elseif name == "ghost" then
    local lift = kv.lift ~= "default" and tonumber(kv.lift) or nil
    local n = rest:match("^lift=[^/]*/([^/]+)/")
    local s, r = tonumber(kv.s), ink.rng(tonumber(kv.seed))
    flora.ghost(lift, function()
      if n == "leaf" or n == "leaflet" or n == "petal" then ink.scale(s) flora[n](r, {}) else flora[n](r, s, {}) end
    end)
    nums(out, ink.get_lift())
  else
    local spec = SPEC[name]
    if not spec then error("bad flora id: " .. id) end
    local o = {}
    for _, key in ipairs(order) do
      if key ~= spec[2] and key ~= "seed" then o[key] = val(kv[key]) end
    end
    local s, r = tonumber(kv[spec[2]]), ink.rng(tonumber(kv.seed))
    if spec[2] == "scale" then ink.scale(s) flora[name](r, o)
    else
      local x, y = flora[name](r, s, o)
      if x then nums(out, x, y) end
    end
  end
  ink.flush()
  return finish(out)
end)

return M
