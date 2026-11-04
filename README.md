Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.

# florilegium (Typst)

Procedurally generated Delft-blue floral page margins for Typst: a port of
[florilegium](https://github.com/kevinraymond/florilegium) (LaTeX/LuaLaTeX, version 1.3).
No image files: every petal, leaf and stem is a parametric curve computed by a wasm
plugin (a Rust port of the original Lua) while the document is compiled, painted like a
watercolor, and drawn as one SVG behind each page. Every page gets its own garden.

Two compositions: **authored** (the default: stems, leaves and flowers placed by hand for
A4 with a 68 mm left margin, every parameter sampled) and **grown** (a plant grown to fit
whatever page layout and free space the text leaves).

## Usage

```typ
#import "@preview/florilegium:0.1.0": *

#set page(paper: "a4")
#show: florilegium.with()          // authored; or mode: "grown"

= Hello
Text beside the garden.
```

From a clone of the repository (`lib/`), import the source instead:
`#import "src/lib.typ": *` (and `--root` the repository if the document is elsewhere), or
install it as a local package (`@local/florilegium:0.1.0`).

The template sets the page margins (default left 68 mm, right 22 mm, top 32 mm,
bottom 30 mm), the header and footer distances, and owns `page.background` (a background
set by the user is replaced; with `garden: false` there is none). Set a header or footer
*after* the `show` rule. The page must have a fixed width and height.

Examples (from `lib/`): `typst compile examples/demo.typ --root .` is the authored
composition with the text of `florilegium.tex`; `examples/grown.typ` is the grown one
with section flowers, blooms, an initial, inline ornaments, aging and a math-theme last
page.

## API

| LaTeX | Typst |
|---|---|
| `\usepackage[opts]{florilegium}` | `#show: florilegium.with(..)` |
| option `authored` / `grown` | `mode: "authored"` (default) / `"grown"` |
| `nomargin` | `garden: false` |
| `novisitors` | `visitors: false` |
| `sections` | `sections: true` (numbered headings of level 1 and 2 get a flower) |
| `aging` | `aging: true` |
| `theme=`, `density=`, `intensity=` | `theme: "flora"` (also `"geometric"`, `"math"`), `density: 1`, `intensity: 1` |
| colors `ink`, `accent` | `ink: rgb("#2456C4")`, `accent: rgb("#B4662F")` |
| `\def\florilegiumseed{7}` | `seed: 7` (`none`: the garden follows the page number) |
| `\florilegiumtextend` | `#textend()` |
| `\florilegiumbloom[motif]` | `#bloom()` / `#bloom(motif: "umbel")` |
| `\florilegiumage{a}` | `#age(0.3)` |
| `\florilegiumtheme{name}` | `#theme("math")` |
| `\florilegiuminitial[4]{L}orem...` | `#initial("L", lines: 4)[orem...]` |
| `\florilegiuminitialfont{f.otf}` | `initial-font: read("f.otf", encoding: none)` on the template, `#initial-font(..)` later, or `font:` on `#initial` |
| `\florilegiumframe[explain]{k}{n}` | `#frame(k, n, explain: false)` |
| `\florilegiumornament[..]{name}` | `#ornament("leaf", size: 1em, ink: none, accent: none, angle: 0deg, seed: auto)` |
| `\florilegiumdraw{name}{size, {opts}}` | `#draw("flower", 1.2, tone: 0.3, ..)` |

Differences of the Typst API from the spec of the port, where the code wins:
`bloom` takes the motif as the named argument `motif:`; `initial` takes the letter and
then the rest of the paragraph (`#initial("L")[orem ...]`).

### Template options

| option | default | meaning |
|---|---|---|
| `mode` | `"authored"` | `"authored"` (A4, 68 mm left margin) or `"grown"` |
| `garden` | `true` | `false`: no margin at all |
| `visitors` | `true` | butterflies, bees, snail, bird |
| `sections` | `false` | a flower beside every numbered heading of level 1 and 2 |
| `aging` | `false` | the season follows the page: spring on the first page, autumn on the last |
| `theme` | `"flora"` | `"flora"`, `"geometric"`, `"math"` (a theme per page with `theme(..)`) |
| `density`, `intensity` | `1` | how much grows; how saturated (density reduces exactly at 1) |
| `ink`, `accent` | Delft blue, copper | colors (RGB; other color spaces are converted) |
| `seed` | `none` | integer >= 1, or the page number |
| `margin` | `(left: 68mm, right: 22mm, top: 32mm, bottom: 30mm)` | as `page(margin: ..)` |
| `twoside` | `false` | `left`/`right` are then the inside/outside margins |
| `headsep`, `headheight`, `footskip` | `8mm`, `12pt`, `30pt` of TeX | LaTeX's distances: the head rule and the foot line (TeX points, converted) |
| `initial-font` | `auto` | bytes of an OpenType/TrueType file for the initials |

Other functions: `#bloom(motif: "")` (a flower beside the line it stands in; motifs:
`lily`, `blossoms`, `cyme`, `umbel`, `bud`, `butterfly`, ... `""` lets the garden choose);
`#age(v)` (0 spring, 0.5 summer, 1 autumn; ignored with `aging: true`);
`#theme(name)`; `#frame(k, n)` (frame `k` of `n` of the growth); `#textend()` (the text ends
here: in grown mode the garden spills into the space below); `#ornament(name, ..)` (an inline
primitive in a box of side `size` on the baseline, depth 0.15 size); `#draw(name, size, ..opts)`
(a primitive drawn at the current point as a zero-size box, `size` in cm, `opts` the
primitive's options: `tone`, `w`, `bend`, `side`, `depth`, ...); `#initial(..)`;
`#initial-font(..)`.

## Judgment calls and differences from the LaTeX package

- **Compiled in Rust, not Lua.** The drawing is a procedure-for-procedure port of the Lua
  (same rng draw order and arithmetic). The Lua is the oracle: the native build is checked
  byte for byte against it (see Testing).
- **Geometry.** The template sets the margins (Typst has no `geometry`); lengths become
  scaled points the way TeX would hold them. A page is mirrored by the same rule as
  `geometry()` (`twoside`).
- **Page number and seed.** The garden is seeded from the logical page number
  (`counter(page)`, LaTeX's `\value{page}`), as in Lua. With `twoside: true` the mirroring
  parity follows the *physical* page, so a reset of the logical counter does not swap sides.
  An explicit `seed:` replaces the page-number seed.
- **No `seed: "random"`.** (`os.time` in LaTeX.) Typst compilation is deterministic; it is
  an error with a message saying so. Give an integer.
- **`draw` default seed.** In LaTeX each draw is the next fork of the page rng, in TeX's
  execution order. Typst has no such order, so the default seed is
  `(seed or 1) * 7919 + physical page * 131 + k * 977` with `k` the document count of draws; an
  explicit `seed:` overrides it. A draw that moves to another page on reflow changes.
  Likewise `ornament` defaults to `n * 7919 + 4243` for the n-th ornament of the document
  and `seed * 7919 + 17` for an explicit one.
- **Initials** have no `\hangafter` in Typst: the paragraph is wrapped beside the tile with
  [droplet](https://typst.app/universe/package/droplet) 0.3.1, the tile being the cap. The
  letter's outline is read from the bundled Latin Modern Roman Bold (or your font); the
  letter is also set transparent over the tile so the text extracts. The tile uses its own
  rng stream (`seed * 7919 + 5003`, `seed` the argument or the document count of initials);
  LaTeX forks whatever page rng TeX last seeded, which cannot be reproduced outside TeX.
- **Captions of the math theme** (formulas, figures' labels) are set by Typst in the
  document's font and Typst's math font, so the glyphs differ from LaTeX's Computer Modern.
  Position, size, rotation, color and opacity are the original's.
- **Lengths (Dimens rounding).** See the notes below: a length is exactly TeX's for short
  decimals in bp, mm, cm, in and TeX pt and rounded otherwise (at most 1 sp, invisible).
- **Footer.** `footer-descent` is set to `footskip - 10pt`, an approximation of the original's.
- **Narrow later margins.** Blooms and garden drawn beyond a narrow margin are clipped, as the original clips its drawing to the column.
- **Density** below 0.03 behaves as 0.03 (the original's floor when `density < 1`); negative values are accepted.
- **`textend` semantics.** `#textend()` ends the paragraph and leaves a block-level mark;
  the page it is on grows with spill, from the height of the material above the mark (LaTeX
  measured `\pagetotal + \pagedepth`; Typst also counts the paragraph spacing).
- **Aging.** A page takes the last `age` / `frame` mark on or before it. With `aging: true`
  the season follows the page and the last page (`counter(page).final()`), and wins over
  `age` marks, as in LaTeX; no second run is needed, Typst converges by itself.
- **Math-theme equations** along the bottom can touch or overlap, as in the original, whose spacing uses a rough width estimate (0.2 cm per character); Typst's widths match TeX's within 0.1 cm.
- **An unknown primitive** is an error (`florilegium: no primitive '..'`).
- **Colors.** The plugin accepts `RRGGBB` and `#RRGGBB`; `ink` and `accent` are `rgb` values.

## Credits

- florilegium by Kevin Raymond (MIT): Kevin Raymond wrote the original LaTeX/Lua package this ports.
- Oklab: Björn Ottosson, "A perceptual color space for image processing" (2020).
- mulberry32: Tommy Ettinger's public-domain generator.
- Wong, Zongker and Salesin, "Computer-generated floral ornament", SIGGRAPH 1998: the idea
  of the grown composition.
- [droplet](https://typst.app/universe/package/droplet) by Eric Biedert (MIT): the
  paragraph wrapped around the initial.
- Latin Modern Roman 10 Bold (c) 2003-2009 B. Jackowski and J. M. Nowacki (GUST
  e-foundry), GUST Font License; bundled in `fonts/`.

## Building

The plugin is committed as `src/florilegium.wasm`. To rebuild it: install the tools of
`mise.toml` (`mise install`; rust with the `wasm32-unknown-unknown` target) and run

```sh
make wasm
```

## Testing

From `lib/`, through `mise exec --` so that the pinned versions run:

```sh
cargo test --release --manifest-path core/Cargo.toml   # oracle comparisons, wasm vs native (the grown family takes ~90 s)
tt run                                                  # tytanic: Typst API tests and reference images
tools/oracle/gen.sh                                     # regenerates the oracle fixtures from ../florilegium (needs lua)
```

Look at rendered pages too: `typst compile examples/demo.typ --root . --format png`.

## Notes

### Lengths: Typst's conversion against TeX's `scan_dimen`

`geometry.to-sp` reproduces `scan_dimen` exactly for any length that is a short decimal in bp, mm, cm, in or TeX pt, and rounds only otherwise. The table below is what plain rounding would give, not what `to-sp` returns for these values.

The core lays the page out in scaled points (sp), the way `florilegium-svg.lua` and the geometry
package do, and the Typst layer converts lengths with `round(bp * 72.27/72 * 65536)`.
TeX's `scan_dimen` (`svg::sp`) truncates the carried remainder instead of rounding it, so
the two disagree by 1 sp (about 5e-7 cm, invisible) for some values. Measured by
`typst_lengths_match_tex` in `core/tests/glue.rs` (Typst `mm` values as `mm * 72/25.4` bp;
Typst's `pt` is TeX's `bp`):

| length | Typst sp | `sp()` sp | diff |
|---|---|---|---|
| 68mm | 12679823 | 12679822 | +1 |
| 22mm | 4102296 | 4102295 | +1 |
| 32mm | 5966975 | 5966975 | 0 |
| 30mm | 5594039 | 5594039 | 0 |
| 8mm | 1491744 | 1491743 | +1 |
| 12pt | 789381 | 789381 | 0 |
| 30pt | 1973453 | 1973452 | +1 |
| a4 (210mm) | 39158276 | 39158276 | 0 |
| letter (8.5in) | 40258437 | 40258437 | 0 |

So a Typst page with the default margins is not byte-identical to `florilegium-svg.lua`'s page in the
last digit of some coordinates; use `florilegium-svg.lua`-style lengths (`Dimens::from_margins` with
`svg::sp`) where byte equality matters.

### The wasm plugin against the native build

`core/tests/wasm.rs` runs `src/florilegium.wasm` (`make wasm`) in wasmi and compares the
SVG of `page` for the `florilegium-svg.lua --seed N` pages, N = 1..8 (default A4 layout), with the
native result. Measured with rust 1.99.1, native glibc libm: 0 differing bytes in all eight
pages (2.6 to 3.1 MB each). Rust's wasm std uses compiler-builtins' libm for `sin`, `cos`,
`powf`, `exp`, ...; none of the seeds tried shows a one-ulp difference after the 1-decimal
rounding of the output, but other seeds could, and any such difference is a libm ulp.

Speed: one A4 page takes about 0.1 s natively and about 1.6 s in the plugin (wasmi interprets
the module), 2.3 s with its 3 MB SVG drawn; `typst compile` caches the call, so it is paid
once per distinct page. Measured (typst 0.15.1, PDF): `examples/demo.typ`, 4 pages, 4.6 s;
`examples/grown.typ`, 4 pages (sections, an initial, the math theme), 8.4 s.

The plugin's CBOR interface (functions, fields, units, caps) is documented in the module
docs of `core/src/plugin.rs`.
