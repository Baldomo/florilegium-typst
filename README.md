Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.

# florilegium (Typst)

A Typst port of florilegium, procedurally generated Delft-blue floral page margins.
This file is a stub: Task 17 expands it into the package manual.

## Notes

### Lengths: Typst's conversion against TeX's `scan_dimen`

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
