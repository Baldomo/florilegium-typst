Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.

# florilegium (Typst)

A Typst port of florilegium, procedurally generated Delft-blue floral page margins.
This file is a stub: Task 17 expands it into the package manual.

## Notes

### Lengths: Typst's conversion against TeX's `scan_dimen`

(`geometry.to-sp` now reproduces `scan_dimen` exactly for any length that is a short decimal in bp, mm, cm, in or TeX pt, and rounds only otherwise; the table below is what plain rounding would give.)

The core lays the page out in scaled points (sp), the way `svg.lua` and the geometry
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

So a Typst page with the default margins is not byte-identical to `svg.lua`'s page in the
last digit of some coordinates; use `svg.lua`-style lengths (`Dimens::from_margins` with
`svg::sp`) where byte equality matters.

### The wasm plugin against the native build

`core/tests/wasm.rs` runs `src/florilegium.wasm` (`make wasm`) in wasmi and compares the
SVG of `page` for the `svg.lua --seed N` pages, N = 1..8 (default A4 layout), with the
native result. Measured with rust 1.99.1, native glibc libm: 0 differing bytes in all eight
pages (2.6 to 3.1 MB each). Rust's wasm std uses compiler-builtins' libm for `sin`, `cos`,
`powf`, `exp`, ...; none of the seeds tried shows a one-ulp difference after the 1-decimal
rounding of the output, but other seeds could, and any such difference is a libm ulp.

Speed: one A4 page takes about 0.1 s natively; the `page` call takes about 1.6 s inside Typst
(wasmi interprets the module) and a page with its 3 MB SVG drawn about 2.3 s in total.
`typst compile` caches the call, so it is paid once per distinct page.

The plugin's CBOR interface (functions, fields, units, caps) is documented in the module
docs of `core/src/plugin.rs`.
