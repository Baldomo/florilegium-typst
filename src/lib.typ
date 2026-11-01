// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
// Procedurally generated Delft-blue floral page margins (a port of florilegium).

#import "garden.typ" as _garden

/// The template: `#show: florilegium.with(..)`. It sets the page's margins, header and
/// footer distances and `page.background`, which it owns: a background set by the user
/// is replaced (with `garden: false` it is none).
///
/// - mode: "authored" (the hand-placed composition, for A4 with a 68 mm left margin) or "grown".
/// - theme: "flora", "geometric" or "math".
/// - seed: none (the garden follows the page number) or an integer >= 1.
/// - margin: as `page(margin: ..)`; with `twoside: true` the dictionary's `left`/`right`
///   are the inside/outside margins.
/// - headsep, headheight, footskip: LaTeX's distances. The defaults are TeX points
///   (12pt and 30pt of LaTeX), converted since Typst's `pt` is the bp (1/72 in).
///   `headsep` is the header's ascent; `headheight` only matters to the garden's geometry.
/// - sections, aging, initial-font: stored for the section, aging and initial features.
#let florilegium(
  mode: "authored",
  garden: true,
  visitors: true,
  sections: false,
  aging: false,
  theme: "flora",
  density: 1,
  intensity: 1,
  ink: rgb("#2456C4"),
  accent: rgb("#B4662F"),
  seed: none,
  margin: (left: 68mm, right: 22mm, top: 32mm, bottom: 30mm),
  twoside: false,
  headsep: 8mm,
  headheight: 12pt * 72 / 72.27,
  footskip: 30pt * 72 / 72.27,
  initial-font: auto,
  body,
) = {
  let cfg = _garden.cfg(
    mode: mode, garden: garden, visitors: visitors, sections: sections, aging: aging,
    theme: theme, density: density, intensity: intensity, ink: ink, accent: accent,
    seed: seed, margin: margin, twoside: twoside, headsep: headsep, headheight: headheight,
    footskip: footskip, initial-font: initial-font,
  )
  let margin = if twoside and type(margin) == dictionary {
    let m = margin
    if "left" in m { m.inside = m.remove("left") }
    if "right" in m { m.outside = m.remove("right") }
    m
  } else { margin }
  set page(
    margin: margin,
    header-ascent: headsep,
    footer-descent: calc.max(0pt, footskip - 10pt),
    background: _garden.background(cfg),
  )
  body
}
