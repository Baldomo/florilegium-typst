-- Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
-- Dumps luaotfload's loadshapes of lmroman10-bold.otf for the characters of the initial
-- (U+0020..U+007E, U+00C0..U+00FF) to core/tests/fixtures/lmroman10-bold.shapes.
-- It needs luaotfload's font loader, so run it inside LuaHBTeX, from lib/:
--   ~/.TinyTeX/bin/x86_64-linux/lualatex -interaction=batchmode -output-directory=/tmp \
--     '\directlua{dofile("tools/oracle/shapes.lua")}\end'
-- (TeX Live 2025, luaotfload 3.29, fontloader-2023-12-28; the font is read from the TeX tree,
-- a byte-identical copy of fonts/lmroman10-bold.otf).
-- Format: "units N", then per glyph "glyph U x1 y1 x2 y2" (boundingbox) or "glyph U nobox", then
-- one line per segment: "m x y", "l x y", "c x1 y1 x2 y2 x3 y3", "q x1 y1 x2 y2"; "end" closes the
-- glyph; "glyph U nosegments" for a glyph without outline. Numbers are %.17g.
local out = "core/tests/fixtures/lmroman10-bold.shapes"
local path = assert(kpse.find_file("lmroman10-bold.otf", "opentype fonts"))
local shapes = fonts.handlers.otf.readers.loadshapes(path)
local byuni, dup = {}, {}
for _, gl in pairs(shapes.glyphs or {}) do
  if type(gl.unicode) == "number" then
    if byuni[gl.unicode] then dup[#dup + 1] = gl.unicode else byuni[gl.unicode] = gl end
  end
end
local function g(x) return ("%.17g"):format(x) end
local lines = { "units " .. g(shapes.units or 1000) }
local function want(u) return (u >= 0x20 and u <= 0x7E) or (u >= 0xC0 and u <= 0xFF) end
local us = {}
for u in pairs(byuni) do if want(u) then us[#us + 1] = u end end
table.sort(us)
for _, u in ipairs(us) do
  local gl = byuni[u]
  local bb = gl.boundingbox
  local head = "glyph " .. u
  if bb then head = head .. " " .. g(bb[1]) .. " " .. g(bb[2]) .. " " .. g(bb[3]) .. " " .. g(bb[4]) else head = head .. " nobox" end
  if not gl.segments then head = head .. " nosegments" end
  lines[#lines + 1] = head
  if gl.segments then
    for _, sg in ipairs(gl.segments) do
      local t = { sg[#sg] }
      for i = 1, #sg - 1 do t[#t + 1] = g(sg[i]) end
      lines[#lines + 1] = table.concat(t, " ")
    end
    lines[#lines + 1] = "end"
  end
end
local f = assert(io.open(out, "w"))
f:write(table.concat(lines, "\n"), "\n")
f:close()
texio.write_nl("shapes: " .. #us .. " glyphs, duplicates of unicode: " .. #dup)
