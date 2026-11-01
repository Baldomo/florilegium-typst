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

-- ---------------------------------------------------------------- geometric
-- the primitives of theme-geometric.lua, as the flora family drives flora's
local geo = require("theme-geometric")
local GEO = {
  { "flower", "s", { 1.2, 0.16, 2.4 }, { petals = { 5, 6, 8 }, style = { 1, 2 }, tone = { 0.2, 0.5 } } },
  { "lily", "s", { 1.1, 0.21, 2.2 }, { tilt = { 0.3, 0.7 }, tone = { 0.25, 0.45 } } },
  { "blossom", "s", { 0.3, 0.17, 0.8 }, { tone = { 0.2, 0.5 }, eye = { "0.9,0.2,0.1" } } },
  { "floret", "s", { 0.1, 0.05, 0.5 }, { tone = { 0.3, 0.4 }, eye = { "0.9,0.2,0.1" } } },
  { "bud", "s", { 0.6, 0.33, 1.2 }, { tone = { 0.25, 0.5 } } },
  { "bell", "s", { 1, 0.33, 2 }, { tone = { 0.25, 0.5 } } },
  { "leaf", "scale", { 2, 0.35, 4 }, { depth = { 0, 0.6 }, w = { 0.15, 0.3 }, lobes = { 2, 6 }, bend = { 0.3, -0.3 }, tone = { 0.2, 0.45 } } },
  { "leaflet", "scale", { 0.6, 0.35, 1.5 }, { w = { 0.1, 0.3 }, bend = { 0.3, -0.3 }, tone = { 0.15, 0.4 } } },
  { "petal", "scale", { 1.2, 0.35, 3 }, { w = { 0.2, 0.5 }, tone = { 0.25, 0.5 } } },
  { "cyme", "s", { 2, 0.33, 3 }, { side = { 1, -1 }, fork = { "false", "true" } } },
  { "sprig", "s", { 2, 0.35, 3 }, { depth = { 2, 5 } } },
  { "umbel", "s", { 1.2, 0.28, 2.4 }, {} },
  { "seedhead", "s", { 1, 0.15, 2 }, { side = { 1, -1 }, drift = { 0, 5 } } },
  { "tendril", "s", { 1, 0.16, 2 }, {} },
  { "rosette", "s", { 0.3, 0.13, 0.6 }, { color = { "0.2,0.5,0.3" } } },
  { "hedera", "s", { 0.36, 0.13, 0.8 }, { color = { "0.2,0.5,0.3" } } },
  { "butterfly", "s", { 2, 0.34, 4 }, { open = { 0.3, 1 }, tone = { 0.2, 0.45 } } },
  { "bee", "s", { 1, 0.28, 2 }, {} },
  { "snail", "s", { 1, 0.19, 2 }, {} },
  { "bird", "s", { 1, 0.33, 2 }, { facing = { -1, 1 }, tone = { 0.25, 0.45 } } },
  { "crystal", "s", { 1, 0.3, 2 }, { k = { 1, 2, 3, 4, 5 } } },
}
local GEO_SPEC = {}
for _, r in ipairs(GEO) do GEO_SPEC[r[1]] = r end
local GEO_GHOSTED = { "flower", "lily", "blossom", "floret", "bud", "bell", "leaf", "leaflet", "petal", "cyme", "sprig", "umbel",
  "seedhead", "butterfly", "bee", "snail", "bird", "crystal" }
local GEO_GHOST_SIZES = { flower = 1.2, lily = 1.1, blossom = 0.3, floret = 0.1, bud = 0.6, bell = 1, leaf = 2, leaflet = 0.6, petal = 1.2,
  cyme = 2, sprig = 2, umbel = 1.2, seedhead = 1, butterfly = 2, bee = 1, snail = 1, bird = 1, crystal = 1 }

M.register("geometric", function()
  local ids = {}
  local function add(id) ids[#ids + 1] = id end
  for _, r in ipairs(GEO) do
    for _, s in ipairs(r[3]) do
      for k = 1, 4 do add(("geometric/%s/%s=%s/seed=%d"):format(r[1], r[2], tostring(s), k)) end
    end
    for _, key in ipairs(sorted_keys(r[4])) do
      for _, v in ipairs(r[4][key]) do
        for k = 1, 4 do add(("geometric/%s/%s=%s/seed=%d/%s=%s"):format(r[1], r[2], tostring(r[3][1]), k, key, tostring(v))) end
      end
    end
  end
  for _, lift in ipairs(LIFTS) do
    for _, name in ipairs(GEO_GHOSTED) do
      for k = 1, 3 do add(("geometric/ghost/lift=%s/%s/s=%s/seed=%d"):format(lift, name, tostring(GEO_GHOST_SIZES[name]), k)) end
    end
  end
  for _, st in ipairs(STEMS) do add("geometric/stem/" .. st) end
  return ids
end, function(id)
  local name, rest = id:match("^geometric/([^/]+)/(.*)$")
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
  if name == "stem" then
    local pts = {}
    if kv.path == "bytes" then for i = 0, 80 do pts[#pts + 1] = { math.sin(i / 9), i / 4 } end
    else pts = { { 0, 0 }, { 1, 0 }, { 2, 1 }, { 3, 1 }, { 3, 2 }, { 2, 3 }, { 2, 4 }, { 1, 5 }, { 0, 5 }, { 0, 6 } } end
    if kv.scale then ink.scale(tonumber(kv.scale)) end
    local c = kv.c and val(kv.c) or nil
    geo.stem(pts, tonumber(kv.w0), tonumber(kv.w1), c, kv.op and tonumber(kv.op) or nil, kv.tip and tonumber(kv.tip) or nil)
  elseif name == "ghost" then
    local lift, n = rest:match("^lift=([^/]+)/([^/]+)/")
    local s, r = tonumber(kv.s), ink.rng(tonumber(kv.seed))
    flora.ghost(lift ~= "default" and tonumber(lift) or nil, function()
      if n == "leaf" or n == "leaflet" or n == "petal" then ink.scale(s) geo.flora[n](r, {}) else geo.flora[n](r, s, {}) end
    end)
    nums(out, ink.get_lift())
  else
    local spec = GEO_SPEC[name]
    if not spec then error("bad geometric id: " .. id) end
    local o = {}
    for _, key in ipairs(order) do
      if key ~= spec[2] and key ~= "seed" then o[key] = val(kv[key]) end
    end
    local s, r = tonumber(kv[spec[2]]), ink.rng(tonumber(kv.seed))
    if spec[2] == "scale" then ink.scale(s) geo.flora[name](r, o)
    else
      local x, y = geo.flora[name](r, s, o)
      if x then nums(out, x, y) end
    end
  end
  ink.flush()
  return finish(out)
end)

-- ---------------------------------------------------------------- figures
local figs = require("figures")
local FIGS, FAMS = figs.figures, figs.families
local FIG_RS = { "0.5", "1.2" }

local function pnums(out, label, p)
  local t = { label }
  for i = 1, #p do
    local v = p[i]
    if v == true then v = 1 elseif v == false then v = 0 end
    t[#t + 1] = G(v)
  end
  out[#out + 1] = table.concat(t, " ")
end

local function all_texts()
  local texts = {}
  local r = ink.rng(1)
  for _, name in ipairs(figs.figure_order) do
    local f = FIGS[name]
    for _ = 1, 50 do texts[#texts + 1] = { name, f.caption(f.pick(r)) } end
  end
  for _, name in ipairs(figs.family_order) do
    local f = FAMS[name]
    for _ = 1, 5 do
      local p = f.pick(r)
      texts[#texts + 1] = { name, f.title(p) }
      for _, v in ipairs(f.values) do texts[#texts + 1] = { name, f.label(v) } end
    end
  end
  return texts
end

M.register("figures", function()
  local ids = {}
  local function add(id) ids[#ids + 1] = id end
  for _, name in ipairs(figs.figure_order) do
    for _, R in ipairs(FIG_RS) do
      for k = 1, 4 do add(("figures/%s/R=%s/seed=%d"):format(name, R, k)) end
    end
    add(("figures/%s/R=1.2/seed=1/lift=0.4"):format(name))
  end
  for _, name in ipairs(figs.family_order) do
    for i = 1, #FAMS[name].values do
      for _, R in ipairs(FIG_RS) do
        for k = 1, 4 do add(("figures/family-%s/v=%d/R=%s/seed=%d"):format(name, i, R, k)) end
      end
    end
  end
  for _, id in ipairs({
    "helper/phyllotaxis/R=1.2/seed=1", "helper/phyllotaxis/R=0.1/seed=2", "helper/phyllotaxis/R=1.2/seed=3/side=-1/drift=3",
    "helper/phyllotaxis/R=1.2/seed=4/lift=0.4", "helper/maurer/R=1/seed=1", "helper/maurer/R=1/seed=2",
    "helper/fay/s=2/seed=1", "helper/fay/s=0.2/seed=2", "helper/fay/s=2/seed=3/open=0.5/tone=0.4",
    "helper/compass/s=1/seed=1", "helper/compass/s=0.5/seed=1/color=0.2,0.5,0.3", "helper/vogel/N=12/R=2/spin=0.3",
    "helper/vogel/N=150/R=0.8/spin=4", "hulls", "captions", "caption_width" }) do add("figures/" .. id) end
  return ids
end, function(id)
  local rest = id:match("^figures/(.*)$")
  local out = { id }
  local kv = {}
  for key, v in rest:gmatch("([^/=]+)=([^/]*)") do kv[key] = v end
  ink.set_transform({ ink.CM, 0, 0, ink.CM, 0, 0 })
  if kv.lift then ink.set_lift(tonumber(kv.lift)) end
  local name = rest:match("^([^/]+)")
  if rest == "captions" then
    for _, t in ipairs(all_texts()) do out[#out + 1] = t[1] .. "\t" .. t[2] end
    return table.concat(out, "\n", 2) .. "\n"
  elseif rest == "caption_width" then
    for _, t in ipairs(all_texts()) do out[#out + 1] = G(figs.caption_width(t[2])) end
    return table.concat(out, "\n", 2) .. "\n"
  elseif rest == "hulls" then
    for i = 1, #figs.solids do
      local v = figs.solids[i][3]
      local t = { "solid" .. i }
      for _, q in ipairs(v) do t[#t + 1] = G(q[1]) t[#t + 1] = G(q[2]) t[#t + 1] = G(q[3]) end
      out[#out + 1] = table.concat(t, " ")
      local h = figs.hull(i)
      nums(out, #h.faces, #h.edges, h.radius)
      for _, f in ipairs(h.faces) do pnums(out, "n", { f.n[1] + 0.0, f.n[2] + 0.0, f.n[3] + 0.0 }) pnums(out, "f", f.v) end
      for _, e in ipairs(h.edges) do pnums(out, "e", { e.a, e.b, e.f[1], e.f[2] or -1 }) end
    end
    local cube, octa = figs.solids[2][3], figs.cyclic({ { 2, 0, 0 } })
    for _, tr in ipairs({ { cube, 1 / (2 + math.sqrt(2)) }, { cube, 0.5 }, { octa, 1 / 3 }, { figs.solids[4][3], 0.2 }, { figs.solids[5][3], 0.5 } }) do
      local pts = figs.truncate(tr[1], tr[2])
      local t = { "trunc" .. #pts }
      for _, q in ipairs(pts) do t[#t + 1] = G(q[1]) t[#t + 1] = G(q[2]) t[#t + 1] = G(q[3]) end
      out[#out + 1] = table.concat(t, " ")
      local h = figs.hull_of(pts)
      nums(out, #h.faces, #h.edges, h.radius)
    end
    return table.concat(out, "\n", 2) .. "\n"
  elseif name == "helper" then
    local what = rest:match("^helper/([^/]+)/")
    local r = ink.rng(tonumber(kv.seed) or 1)
    if what == "phyllotaxis" then
      local o = {}
      if kv.side then o.side = tonumber(kv.side) end
      if kv.drift then o.drift = tonumber(kv.drift) end
      figs.phyllotaxis(r, tonumber(kv.R), o)
    elseif what == "maurer" then
      figs.maurer(r, tonumber(kv.R))
    elseif what == "fay" then
      figs.fay(r, tonumber(kv.s), { open = tonumber(kv.open), tone = tonumber(kv.tone) })
    elseif what == "compass" then
      local c
      if kv.color then local a, b, d = kv.color:match("^([^,]+),([^,]+),([^,]+)$") c = { tonumber(a), tonumber(b), tonumber(d) } end
      nums(out, figs.compass(r, tonumber(kv.s), { color = c }))
    elseif what == "vogel" then
      for _, q in ipairs(figs.vogel(tonumber(kv.N), tonumber(kv.R), tonumber(kv.spin))) do nums(out, q[1], q[2], q[3], q[4]) end
    else
      error("bad figures id: " .. id)
    end
    nums(out, r.u())
    ink.flush()
    return finish(out)
  end
  local R, r = tonumber(kv.R), ink.rng(tonumber(kv.seed))
  local fam = name:match("^family%-(.*)$")
  if fam then
    local f = FAMS[fam]
    local p = f.pick(r)
    pnums(out, "p", p)
    local v = f.values[tonumber(kv.v)]
    if type(v) == "table" and type(v[1]) ~= "table" then pnums(out, "v", v) elseif type(v) == "number" then nums(out, v) end
    out[#out + 1] = f.label(v)
    f.draw(r, R, v, p)
  else
    local f = FIGS[name]
    if not f then error("bad figures id: " .. id) end
    local p = f.pick(r)
    pnums(out, "p", p)
    out[#out + 1] = f.caption(p)
    f.draw(r, R, p)
  end
  nums(out, r.u())
  ink.flush()
  return finish(out)
end)

-- ---------------------------------------------------------------- grow
-- the geometry of tools/ident.lua; `anim` cases replay the recording as animcheck.lua does
local grow = require("grow")
local GROW_MARKS = { { kind = "heading", arg = 1, y = 20 }, { kind = "heading", arg = 2, y = 14 }, { kind = "bloom", arg = "", y = 9 }, { kind = "bloom", arg = "umbel", y = 6 } }
local GROW_CASES = {
  { name = "summer" },
  { name = "novis", visitors = false },
  { name = "spring", age = 0.05 },
  { name = "autumn-marks", age = 0.95, marks = GROW_MARKS },
  { name = "density-0.4", density = 0.4 },
  { name = "density-0.05", density = 0.05 },
  { name = "density-1", density = 1 },
  { name = "age-0.5", age = 0.5 },
  { name = "mirror", mirror = true, marks = GROW_MARKS },
  { name = "math-marks", age = 0.95, themeonly = true, marks = { { kind = "heading", arg = 1, y = 20 }, { kind = "heading", arg = 2, y = 14 },
    { kind = "bloom", arg = "", y = 9 }, { kind = "bloom", arg = "knot", y = 11 }, { kind = "bloom", arg = "umbel", y = 6 } } },
}
local GROW_SEEDS = { 1, 2, 3, 6, 7 }
-- the grown garden in a theme: prefix, seed, spill, case
local GROW_THEME_CASES = {}
for _, seed in ipairs(GROW_SEEDS) do
  for spill = 0, 1 do
    for _, name in ipairs({ "summer", "autumn-marks", "density-0.4" }) do
      GROW_THEME_CASES[#GROW_THEME_CASES + 1] = { "grow", seed, spill, name }
    end
  end
end
GROW_THEME_CASES[#GROW_THEME_CASES + 1] = { "grow", 1, 0, "mirror" }
GROW_THEME_CASES[#GROW_THEME_CASES + 1] = { "grow", 2, 1, "mirror" }
GROW_THEME_CASES[#GROW_THEME_CASES + 1] = { "grow/anim", 1, 0, "summer" }
GROW_THEME_CASES[#GROW_THEME_CASES + 1] = { "grow/anim", 2, 1, "autumn-marks" }
GROW_THEME_CASES[#GROW_THEME_CASES + 1] = { "grow/anim", 3, 0, "density-0.4" }
GROW_THEME_CASES[#GROW_THEME_CASES + 1] = { "grow/anim", 1, 0, "mirror" }

local GROW_MATH_CASES = {}
for _, name in ipairs({ "summer", "math-marks", "density-0.4" }) do
  for _, seed in ipairs(GROW_SEEDS) do
    for spill = 0, 1 do GROW_MATH_CASES[#GROW_MATH_CASES + 1] = { "grow", seed, spill, name } end
  end
end
for _, c in ipairs({ { 1, 0, "mirror" }, { 2, 1, "mirror" }, { 3, 1, "mirror" }, { 1, 1, "density-0.05" }, { 2, 0, "age-0.5" } }) do
  GROW_MATH_CASES[#GROW_MATH_CASES + 1] = { "grow", c[1], c[2], c[3] }
end
for _, c in ipairs({ { 1, 0, "summer" }, { 2, 1, "math-marks" }, { 3, 0, "density-0.4" }, { 1, 0, "mirror" } }) do
  GROW_MATH_CASES[#GROW_MATH_CASES + 1] = { "grow/anim", c[1], c[2], c[3] }
end

-- the captions a composition returns, one per line, as the Rust side prints them
local function caption_lines(out, caps)
  local function f(v) return v == nil and "nil" or sfmt("%.6f", v) end
  for _, c in ipairs(caps) do
    local col = c.color and (f(c.color[1]) .. "," .. f(c.color[2]) .. "," .. f(c.color[3])) or "nil"
    out[#out + 1] = table.concat({ f(c.x), f(c.y), tostring(c.anchor), c.small and "true" or "nil", f(c.scale), f(c.rotate), col, f(c.opacity), c.text }, " ")
  end
end

M.register("grow", function()
  local ids = {}
  for _, prefix in ipairs({ "grow", "grow/anim" }) do
    for _, seed in ipairs(GROW_SEEDS) do
      for spill = 0, 1 do
        for _, c in ipairs(GROW_CASES) do
          if not c.themeonly then ids[#ids + 1] = ("%s/seed=%d/spill=%d/case=%s"):format(prefix, seed, spill, c.name) end
        end
      end
    end
  end
  for _, spec in ipairs(GROW_THEME_CASES) do
    ids[#ids + 1] = ("%s/seed=%d/spill=%d/case=%s/theme=geometric"):format(spec[1], spec[2], spec[3], spec[4])
  end
  for _, spec in ipairs(GROW_MATH_CASES) do
    ids[#ids + 1] = ("%s/seed=%d/spill=%d/case=%s/theme=math"):format(spec[1], spec[2], spec[3], spec[4])
  end
  return ids
end, function(id)
  local anim, seed, spill, name = false, nil, nil, nil
  local rest = id:match("^grow/anim/(.*)$")
  if rest then anim = true else rest = id:match("^grow/(.*)$") end
  local theme
  seed, spill, name, theme = rest:match("^seed=(%d+)/spill=(%d)/case=([^/]+)/theme=(.+)$")
  if not seed then seed, spill, name = rest:match("^seed=(%d+)/spill=(%d)/case=([^/]+)$") end
  local case
  for _, c in ipairs(GROW_CASES) do if c.name == name then case = c end end
  if not case then error("bad grow id: " .. id) end
  seed, spill = tonumber(seed), tonumber(spill)
  local g = { pw = 21, ph = 29.7, textleft = 6.8, textright = 18.8, ytop = 25.4, ybot = 3.0 }
  g.colright, g.headrule, g.footy = g.textleft - 0.4, g.ytop + 0.7, 1.9
  g.seed, g.spill, g.spilly = seed, spill, spill == 1 and 19.0 or 0
  g.age, g.marks, g.visitors, g.density, g.mirror = case.age, case.marks, case.visitors, case.density, case.mirror
  g.theme = theme
  local out = {}
  if not anim and theme == "math" then
    local r = grow.page(g)
    caption_lines(out, r.captions)
    return finish(out)
  end
  if not anim then
    local r = grow.page(g)
    out[#out + 1] = ("motifs %d stems %d cells %d"):format(r.stats.motifs, r.stats.stems, r.stats.cells)
    for _, k in ipairs(sorted_keys(r.stats.counts)) do out[#out + 1] = ("count %s %d"):format(k, r.stats.counts[k]) end
    return finish(out)
  end
  g.anim = true
  local r = grow.page(g)
  local function frame(t, explain)
    ink.emit(r.rec, function(tag)
      if tag.overlay and not explain then return false end
      local e = tag[1]
      local p = (t - e.t0) / e.dur
      return p >= tag[2] and (tag[3] == nil or p < tag[3])
    end)
  end
  frame(math.huge) frame(r.total * 0.5, true)
  local events, seen = {}, {}
  ink.emit(r.rec, function(tag)
    local e = tag[1]
    if not seen[e] then seen[e] = true events[#events + 1] = e end
    return false
  end)
  out[#out + 1] = "total " .. G(r.total)
  out[#out + 1] = "events " .. #events
  for _, e in ipairs(events) do out[#out + 1] = G(e.t0) .. " " .. G(e.dur) end
  if r.captions then
    caption_lines(out, r.captions)
    for _, c in ipairs(r.captions) do out[#out + 1] = G(c.event.t0) .. " " .. G(c.event.dur) end
  end
  return finish(out)
end)

-- ---------------------------------------------------------------- authored
-- the same geometry; the page writes its own streams and returns nothing
local authored = require("authored")
local AUTH_CASES = {}
for s = 1, 7 do AUTH_CASES[#AUTH_CASES + 1] = { seed = s, name = "plain" } end
for s = 1, 2 do AUTH_CASES[#AUTH_CASES + 1] = { seed = s, name = "novis", visitors = false } end
for s = 1, 2 do AUTH_CASES[#AUTH_CASES + 1] = { seed = s, name = "mirror", mirror = true } end

M.register("authored", function()
  local ids = {}
  for _, c in ipairs(AUTH_CASES) do ids[#ids + 1] = ("authored/seed=%d/case=%s"):format(c.seed, c.name) end
  for s = 1, 4 do ids[#ids + 1] = ("authored/seed=%d/case=plain/theme=geometric"):format(s) end
  ids[#ids + 1] = "authored/seed=1/case=mirror/theme=geometric"
  for s = 1, 2 do ids[#ids + 1] = ("authored/seed=%d/case=plain/theme=math"):format(s) end
  ids[#ids + 1] = "authored/seed=1/case=mirror/theme=math"
  return ids
end, function(id)
  local seed, name, theme = id:match("^authored/seed=(%d+)/case=([^/]+)/theme=(.+)$")
  if not seed then seed, name = id:match("^authored/seed=(%d+)/case=([^/]+)$") end
  local case
  for _, c in ipairs(AUTH_CASES) do if c.seed == tonumber(seed) and c.name == name then case = c end end
  if not case then error("bad authored id: " .. id) end
  local g = { pw = 21, ph = 29.7, textleft = 6.8, textright = 18.8, ytop = 25.4, ybot = 3.0 }
  g.colright, g.headrule, g.footy = g.textleft - 0.4, g.ytop + 0.7, 1.9
  g.seed, g.visitors, g.mirror = case.seed, case.visitors, case.mirror
  g.theme = theme
  local r = authored.page(g)
  local out = {}
  if r and r.captions then caption_lines(out, r.captions) end
  return finish(out)
end)

-- ---------------------------------------------------------------- math
-- theme-math.lua: its page composition's captions, and its flora table by name
local maththeme = require("theme-math")
local MATH_R = { "0.5", "1.2" }
local MATH_FAM_ONLY = {}
do
  local isfig = {}
  for _, n in ipairs(figs.figure_order) do isfig[n] = true end
  for _, n in ipairs(figs.family_order) do if not isfig[n] then MATH_FAM_ONLY[#MATH_FAM_ONLY + 1] = n end end
end
local function csv(t)
  if type(t) == "number" then t = { t } end
  local o = {}
  for i, v in ipairs(t) do o[i] = v == true and "true" or v == false and "false" or sfmt("%.17g", v) end
  return table.concat(o, ",")
end
local function uncsv(s)
  local t = {}
  for v in s:gmatch("[^,]+") do t[#t + 1] = v == "true" or (v ~= "false" and tonumber(v)) end
  return t
end

M.register("math", function()
  local ids = {}
  local function add(id) ids[#ids + 1] = id end
  for _, case in ipairs({ { 1, 0, "summer" }, { 2, 1, "summer" }, { 3, 1, "math-marks" }, { 1, 1, "mirror" }, { 2, 0, "density-0.4" },
      { 3, 0, "density-0.05" }, { 4, 1, "density-0.15" } }) do
    add(("math/captions/seed=%d/spill=%d/case=%s"):format(case[1], case[2], case[3]))
  end
  for _, name in ipairs(figs.figure_order) do
    for _, R in ipairs(MATH_R) do
      for k = 1, 3 do add(("math/%s/R=%s/seed=%d"):format(name, R, k)) end
    end
    for k = 1, 3 do
      add(("math/%s/R=1.2/seed=%d/p=%s"):format(name, k, csv(FIGS[name].pick(ink.rng(50 + k)))))
    end
  end
  for _, name in ipairs(MATH_FAM_ONLY) do
    for _, R in ipairs(MATH_R) do
      for k = 1, 3 do add(("math/%s/R=%s/seed=%d"):format(name, R, k)) end
    end
    for i, v in ipairs(FAMS[name].values) do
      add(("math/%s/R=1.2/seed=%d/vi=%d"):format(name, 1 + i % 3, i))
      add(("math/%s/R=0.5/seed=%d/vi=%d/p=%s"):format(name, 1 + i % 3, i, csv(FAMS[name].pick(ink.rng(60 + i)))))
    end
  end
  for _, c in ipairs({ { "cube", "0,0.3" }, { "cube", "1,0.2" }, { "dodeca", "2,0.3" }, { "dodeca", "3,0.6" }, { "cube", "0,0.45" } }) do
    add(("math/%s/R=1.2/seed=1/vs=%s"):format(c[1], c[2]))
  end
  add("math/rose/R=1.2/seed=1/v=9,4")
  add("math/archimedes/R=1.2/seed=2/v=5")
  for _, R in ipairs(MATH_R) do
    for k = 1, 2 do add(("math/hedera/R=%s/seed=%d"):format(R, k)) end
    add(("math/hedera/R=%s/seed=1/color=0.2,0.5,0.3"):format(R))
  end
  return ids
end, function(id)
  local name, rest = id:match("^math/([^/]+)/(.*)$")
  local grow = require("grow")
  if name == "captions" then
    local seed, spill, case = rest:match("^seed=(%d+)/spill=(%d)/case=(.+)$")
    local g = { pw = 21, ph = 29.7, textleft = 6.8, textright = 18.8, ytop = 25.4, ybot = 3.0 }
    g.colright, g.headrule, g.footy = g.textleft - 0.4, g.ytop + 0.7, 1.9
    g.seed, g.spill = tonumber(seed), tonumber(spill)
    g.spilly = g.spill == 1 and 19.0 or 0
    local want
    for _, c in ipairs(GROW_CASES) do if c.name == case then want = c end end
    if case == "density-0.15" then want = { density = 0.15 } end
    if not want then error("bad math id: " .. id) end
    g.age, g.marks, g.density, g.mirror = want.age, want.marks, want.density, want.mirror
    local r = maththeme.page(g, grow)
    local out = {}
    caption_lines(out, r.captions)
    return finish(out)
  end
  local out = { id }
  local kv = {}
  for key, v in rest:gmatch("([^/=]+)=([^/]*)") do kv[key] = v end
  ink.set_transform({ ink.CM, 0, 0, ink.CM, 0, 0 })
  local r = ink.rng(tonumber(kv.seed))
  local o
  if kv.vs then
    local a, b = kv.vs:match("^(%d),(.+)$")
    local cf, df = FAMS.cube.values, FAMS.dodeca.values
    local solid = ({ [0] = cf[1][1], cf[4][1], df[1][1], df[4][1] })[tonumber(a)]
    kv.v = nil
    o = { v = { solid, tonumber(b), "x" } }
  end
  if kv.p or kv.v or kv.vi or kv.color then
    o = { p = kv.p and uncsv(kv.p), v = kv.v and uncsv(kv.v), color = kv.color and uncsv(kv.color) }
    if kv.vi then o.v = FAMS[name].values[tonumber(kv.vi)] end
    if o.v and kv.v and FAMS[name] and type(FAMS[name].values[1]) == "number" then o.v = o.v[1] end
  end
  local fn = maththeme.flora[name]
  if not fn then error("bad math id: " .. id) end
  local x, y = fn(r, tonumber(kv.R), o)
  if x then nums(out, x, y) end
  nums(out, r.u())
  ink.flush()
  return finish(out)
end)

-- ---------------------------------------------------------------- initial
-- cases.lua runs under plain Lua, so luaotfload's reader is replaced by the dump of
-- tools/oracle/shapes.lua (core/tests/fixtures/lmroman10-bold.shapes).
local initial_dump = (arg and arg[0] or ""):match("^(.*)/[^/]*$") or "."
initial_dump = initial_dump .. "/../../core/tests/fixtures/lmroman10-bold.shapes"
local function read_shapes()
  local f = assert(io.open(initial_dump, "r"))
  local shapes, cur = { glyphs = {} }, nil
  for line in f:lines() do
    local w = {}
    for t in line:gmatch("%S+") do w[#w + 1] = t end
    if w[1] == "units" then shapes.units = tonumber(w[2])
    elseif w[1] == "glyph" then
      cur = { unicode = math.tointeger(tonumber(w[2])), segments = {} }
      if w[3] ~= "nobox" then cur.boundingbox = { tonumber(w[3]), tonumber(w[4]), tonumber(w[5]), tonumber(w[6]) } end
      if w[#w] == "nosegments" then cur.segments = nil end
      shapes.glyphs[#shapes.glyphs + 1] = cur
    elseif w[1] == "end" then cur = nil
    else
      local sg = {}
      for i = 2, #w do sg[#sg + 1] = tonumber(w[i]) end
      sg[#sg + 1] = w[1]
      cur.segments[#cur.segments + 1] = sg
    end
  end
  f:close()
  return shapes
end
kpse = { find_file = function(name) return name end }
fonts = { handlers = { otf = { readers = { loadshapes = function() return read_shapes() end } } } }
local initial = require("initial")

M.register("initial", function()
  local ids = {}
  for _, letter in ipairs({ "A", "M", "Q", "R", "W", "g", "\xC3\x9F" }) do
    for _, S in ipairs({ "1.4", "2.1" }) do
      for k = 1, 2 do ids[#ids + 1] = ("initial/%s/S=%s/seed=%d"):format(letter, S, k) end
    end
  end
  ids[#ids + 1] = "initial/\xE2\x82\xAC/S=1.4/seed=1"
  return ids
end, function(id)
  local letter, S, seed = id:match("^initial/(.-)/S=([^/]*)/seed=([^/]*)$")
  local out = { id }
  ink.set_transform({ ink.CM, 0, 0, ink.CM, 0, 0 })
  local r = ink.rng(tonumber(seed))
  local ok, err = initial.draw(r, tonumber(S), letter)
  if not ok then return id .. "\nerror " .. err .. "\n" end
  nums(out, r.u())
  ink.flush()
  return finish(out)
end)

-- ---------------------------------------------------------------- page glue and SVG
-- florilegium.lua and svg.lua under the TeX stubs. An id after the family is a list of
-- `key=value` or bare `flag` segments, which become `--key value` / `--flag` for svg.lua's
-- own command line parser: `svg/seed=7/theme=math/twoside` is `svg.lua --seed 7 --theme math --twoside`.
node.id = function(name) return name end
node.subtype = function(name) return name end
local fl = require("florilegium")

local function svg_argv(rest)
  local argv = {}
  for seg in rest:gmatch("[^/]+") do
    local k, v = seg:match("^([^=]+)=(.*)$")
    if k then argv[#argv + 1] = "--" .. k argv[#argv + 1] = v else argv[#argv + 1] = "--" .. seg end
  end
  return argv
end

-- svg.lua's main, with its standard streams captured; hook(page results) sees what page() returns
local function svg_main(argv, hook)
  local buf = {}
  local so, se = io.stdout, io.stderr
  local orig = svg.page
  io.stdout = { write = function(_, s) buf[#buf + 1] = s end }
  io.stderr = { write = function() end }
  if hook then svg.page = function(o) local a, b, c = orig(o) hook(a, b, c) return a, b, c end end
  local ok, code = pcall(svg.main, argv)
  io.stdout, io.stderr, svg.page = so, se, orig
  if not ok then error(code, 0) end
  if code ~= 0 then error("svg.lua failed: " .. table.concat(argv, " "), 0) end
  return table.concat(buf)
end

-- what tools/ornament.lua does with the streams of one primitive
local function standalone(streams)
  local s = table.concat(streams, "\n")
  local x0, y0, x1, y1 = math.huge, math.huge, -math.huge, -math.huge
  for x, y in s:gmatch("(%-?[%d.]+) (%-?[%d.]+) [mlc]") do
    x, y = tonumber(x), tonumber(y)
    x0, x1, y0, y1 = math.min(x0, x), math.max(x1, x), math.min(y0, y), math.max(y1, y)
  end
  local pad = 1
  local parts = svg.translate(s, {}, "o")
  return sfmt('<svg xmlns="http://www.w3.org/2000/svg" viewBox="%.1f %.1f %.1f %.1f"><g transform="matrix(1 0 0 -1 0 %.1f)" stroke-linecap="round" stroke-linejoin="round">\n%s\n</g></svg>\n',
    x0 - pad, -y1 - pad, x1 - x0 + 2 * pad, y1 - y0 + 2 * pad, 0, table.concat(parts, "\n"))
end

local SVG_CASES = {}
for s = 1, 8 do SVG_CASES[#SVG_CASES + 1] = "seed=" .. s end
for _, c in ipairs({ "authored/seed=3", "page=3", "seed=2/season=spring", "seed=2/season=autumn", "seed=2/season=0.3",
    "seed=2/theme=geometric", "seed=2/theme=math", "seed=2/density=0.4", "seed=2/density=0.05", "seed=2/intensity=0.5",
    "seed=2/intensity=0", "seed=2/novisitors", "seed=2/twoside/page=2", "seed=2/textend=120mm", "seed=2/nohead",
    "seed=2/ink=5B2A1F/accent=2F6B3A", "seed=2/paper=letter", "seed=2/paper=a5", "seed=2/paper=160x240",
    "seed=2/left=50mm/right=30mm", "seed=2/background=FFFDF5" }) do
  SVG_CASES[#SVG_CASES + 1] = c
end
local CAPTION_CASES = { "seed=1/theme=math", "seed=3/theme=math", "seed=2/theme=math/twoside/page=2",
  "seed=2/theme=math/intensity=0.5", "seed=2", "seed=1/authored" }

local FRAME_OPTS = {
  summer = {}, spring = { "season=0.05" }, autumn = { "season=0.95" }, geometric = { "theme=geometric" },
  ["geometric-autumn-mirrored"] = { "theme=geometric", "season=0.95", "twoside", "page=2" },
  ["density-0.4"] = { "density=0.4" }, ["density-0.05"] = { "density=0.05" }, math = { "theme=math" },
  ["math-mirrored-density-0.3"] = { "theme=math", "twoside", "page=2", "density=0.3" },
}
local FRAME_ORDER = { "summer", "spring", "autumn", "geometric", "geometric-autumn-mirrored", "density-0.4",
  "density-0.05", "math", "math-mirrored-density-0.3" }

local ORN_NAMES = { "leaf", "leaflet", "petal", "bud", "bell", "sprig", "cyme", "umbel", "tendril", "flower", "blossom",
  "floret", "lily", "seedhead", "butterfly", "bee", "snail", "bird", "sparkle", "rosette", "hedera" }

M.register("svg", function()
  local ids = {}
  for _, c in ipairs(SVG_CASES) do ids[#ids + 1] = "svg/" .. c end
  return ids
end, function(id)
  return svg_main(svg_argv(id:match("^svg/(.*)$")))
end)

M.register("captions", function()
  local ids = {}
  for _, c in ipairs(CAPTION_CASES) do ids[#ids + 1] = "captions/" .. c end
  return ids
end, function(id)
  local more
  svg_main(svg_argv(id:match("^captions/(.*)$")), function(_, _, m) more = m end)
  local out = {}
  for _, c in ipairs(more.captions) do
    out[#out + 1] = sfmt("%.6f %.6f %s %.6f %.6f %s %.6f %s", c.x, c.y, c.anchor, c.size, c.rotate, c.color, c.opacity, c.tex)
  end
  local st = more.stats
  if st then
    out[#out + 1] = ("motifs %d stems %d cells %d"):format(st.motifs, st.stems, st.cells)
    for _, k in ipairs(sorted_keys(st.counts)) do out[#out + 1] = ("count %s %d"):format(k, st.counts[k]) end
  end
  return table.concat(out, "\n") .. "\n"
end)

M.register("frame", function()
  local ids = {}
  for _, name in ipairs(FRAME_ORDER) do
    for _, k in ipairs({ 1, 4, 8 }) do ids[#ids + 1] = ("frame/case=%s/k=%d/n=8"):format(name, k) end
  end
  for _, name in ipairs({ "summer", "autumn", "math" }) do ids[#ids + 1] = ("frame/case=%s/k=4/n=8/explain"):format(name) end
  return ids
end, function(id)
  local name, k, n, tail = id:match("^frame/case=([^/]+)/k=(%d+)/n=(%d+)(.*)$")
  local explain = tail == "/explain"
  local opts = FRAME_OPTS[name]
  if not opts then error("bad frame id: " .. id) end
  fl.frame(tonumber(k), tonumber(n), explain and "explain" or nil)
  return svg_main(svg_argv("seed=2/" .. table.concat(opts, "/")))
end)

-- primitives drawn from the glue, as the TeX side calls them
local function glue_env()
  tex.dimen = { ["pgf@pt@x"] = 0, ["pgf@pt@y"] = 0 }
  tex.error = function(m) error(m, 0) end
  local pgf = { ["pgf@pt@aa"] = "1", ["pgf@pt@ab"] = "0", ["pgf@pt@ba"] = "0", ["pgf@pt@bb"] = "1" }
  token = { get_macro = function(n) return pgf[n] end }
end
local function kv_of(rest)
  local kv = {}
  for key, v in rest:gmatch("([^/=]+)=([^/]*)") do kv[key] = v end
  return kv
end

M.register("ornament", function()
  local ids = {}
  for _, name in ipairs(ORN_NAMES) do
    ids[#ids + 1] = ("ornament/%s/size=786432/angle=0/seed=3"):format(name)
    ids[#ids + 1] = ("ornament/%s/size=1500000/angle=20/count=1"):format(name)
  end
  ids[#ids + 1] = "ornament/crystal/size=786432/angle=0/seed=5/theme=geometric"
  ids[#ids + 1] = "ornament/flower/size=786432/angle=10/seed=2/theme=geometric"
  ids[#ids + 1] = "ornament/rose/size=786432/angle=0/seed=2/theme=math"
  ids[#ids + 1] = "ornament/flower/size=786432/angle=0/count=4"
  ids[#ids + 1] = "ornament/bird/size=786432/angle=-15/seed=9/ink=5B2A1F/accent=2F6B3A"
  return ids
end, function(id)
  local name, rest = id:match("^ornament/([^/]+)/(.*)$")
  local kv = kv_of(rest)
  glue_env()
  fl.theme = kv.theme
  local count = tonumber(kv.count) or 1
  for _ = 1, count do
    common.reset()
    fl.ornament(name, tonumber(kv.size), kv.ink or "", kv.accent or "", tonumber(kv.angle), tonumber(kv.seed))
  end
  fl.theme = nil
  return standalone({ common.streams() })
end)

M.register("draw", function()
  local ids = {}
  for _, name in ipairs(ORN_NAMES) do ids[#ids + 1] = ("draw/%s/s=1.2/seed=3"):format(name) end
  for _, c in ipairs({ "draw/leaf/s=1.8/seed=2", "draw/cyme/s=1.2/seed=4/side=-1", "draw/seedhead/s=1/seed=4/drift=0",
      "draw/bee/s=1/seed=4/trail=false", "draw/crystal/s=1.2/seed=3/theme=geometric",
      "draw/flower/s=1.2/seed=3/theme=geometric", "draw/hedera/s=1/seed=3/theme=geometric",
      "draw/rose/s=1.2/seed=2/theme=math", "draw/hedera/s=1/seed=2/theme=math",
      "draw/ghost/inner=flower/s=1.4/seed=3", "draw/ghost/inner=lily/s=1.2/seed=3/lift=0.4",
      "draw/ghost/inner=crystal/s=1.2/seed=3/theme=geometric", "draw/flower/s=1.2/seed=3/ink=5B2A1F/accent=2F6B3A" }) do
    ids[#ids + 1] = c
  end
  return ids
end, function(id)
  local name, rest = id:match("^draw/([^/]+)/(.*)$")
  local kv = kv_of(rest)
  glue_env()
  fl.theme = kv.theme
  fl.seed(1, tonumber(kv.seed))
  local o = { lift = tonumber(kv.lift), side = tonumber(kv.side), drift = tonumber(kv.drift) }
  if kv.trail == "false" then o.trail = false end
  if kv.ink or kv.accent then
    local function hexc(h) return tonumber(h:sub(1, 2), 16) / 255, tonumber(h:sub(3, 4), 16) / 255, tonumber(h:sub(5, 6), 16) / 255 end
    local ir, ig, ib = hexc(kv.ink or "2456C4")
    local ar, ag, ab = hexc(kv.accent or "B4662F")
    fl.colors(ir, ig, ib, ar, ag, ab)
  end
  if name == "ghost" then
    o.s = tonumber(kv.s)
    fl.draw("ghost", kv.inner, o)
  else
    fl.draw(name, tonumber(kv.s), o)
  end
  fl.theme = nil
  return standalone({ common.streams() })
end)

return M
