Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.

# Changelog

## Unreleased

Follows florilegium 1.3.

- `florilegium-svg --bleed LEN`: a grown margin that reaches the paper's edge runs on past
  it by LEN, and the SVG grows by LEN on every side, for printing to be trimmed. The Typst
  template and the plugin do not take it.
- The oracle loads florilegium 1.3's renamed Lua files (`florilegium-<part>.lua`); the
  comments and documentation name them so. The drawing is unchanged.

## 0.1.0

First release: a Typst port of florilegium 1.2 (LaTeX/LuaLaTeX).

- Rust core, a procedure-for-procedure port of the Lua, compiled to a wasm plugin and
  checked byte for byte against the Lua (oracle tests).
- Template `florilegium(..)`: authored and grown compositions, themes (flora, geometric,
  math), seeds, density, intensity, visitors, section flowers, aging, twoside.
- `bloom`, `theme`, `age`, `frame`, `textend`, `ornament`, `draw`, `initial`, `initial-font`.
- The plugin accepts colors as `RRGGBB` and `#RRGGBB`.
