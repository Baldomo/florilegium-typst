Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.

# Changelog

## 0.1.0

First release: a Typst port of florilegium 1.2 (LaTeX/LuaLaTeX).

- Rust core, a procedure-for-procedure port of the Lua, compiled to a wasm plugin and
  checked byte for byte against the Lua (oracle tests).
- Template `florilegium(..)`: authored and grown compositions, themes (flora, geometric,
  math), seeds, density, intensity, visitors, section flowers, aging, twoside.
- `bloom`, `theme`, `age`, `frame`, `textend`, `ornament`, `draw`, `initial`, `initial-font`.
- The plugin accepts colors as `RRGGBB` and `#RRGGBB`.
