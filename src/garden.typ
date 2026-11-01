// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
// The page garden: the plugin's SVG plus its captions, laid over the paper.

#import "core.typ"
#import "geometry.typ"

/// The settings of a document's garden: the template's named parameters, checked.
/// `sections`, `aging` and `initial-font` are stored for the layers above (marks, aging
/// and the initial) and do not change the page itself.
#let cfg(
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
) = {
  if seed != none and (type(seed) != int or seed < 1) {
    panic("florilegium: seed must be a positive integer")
  }
  (
    mode: mode, garden: garden, visitors: visitors, sections: sections, aging: aging,
    theme: theme, density: density, intensity: intensity, ink: ink, accent: accent,
    seed: seed, margin: margin, twoside: twoside, headsep: headsep, headheight: headheight,
    footskip: footskip, initial-font: initial-font,
  )
}

#let _hex(c) = c.rgb().to-hex().slice(0, 7)

/// The plugin's `page` result for the page being laid out. Needs a context. The margins
/// are those of this page as Typst lays it out (`page.margin`, with inside and outside
/// resolved for the physical page: Typst's two-sided margins follow the physical page
/// number, not the page counter), sent as the left margin of a one-sided page; the core
/// mirrors the garden when the right margin is the wider. The seed of a document without
/// `seed:` is the logical page number.
/// `marks`, `theme` (the effective theme), `age`, `frame` and `spilly` are what the marks
/// layer supplies per page.
#let debug(cfg, marks: (), theme: auto, age: none, frame: none, spilly: none) = {
  let (w, h) = (page.width, page.height)
  if w == auto or h == auto { panic("florilegium: the page needs a fixed width and height") }
  let margins = geometry.resolve-margin(page.margin, w, h, even: calc.even(here().page()))
  let number = counter(page).get().first()
  core.page((
    page: calc.clamp(number, 1, 1000000),
    twoside: false,
    dimens: geometry.dimens(w, h, margins, cfg.headheight, cfg.headsep, cfg.footskip),
    seed: cfg.seed,
    mode: cfg.mode,
    theme: if theme == auto { cfg.theme } else { theme },
    age: age,
    marks: marks,
    density: cfg.density,
    intensity: cfg.intensity,
    visitors: cfg.visitors,
    ink: _hex(cfg.ink),
    accent: _hex(cfg.accent),
    spilly: spilly,
    frame: frame,
  ))
}

#let _caption(c) = {
  let anchor = c.anchor
  let ax = if "west" in anchor { 0 } else if "east" in anchor { 1 } else { 0.5 }
  let ay = if "north" in anchor { 0 } else if "south" in anchor { 1 } else { 0.5 }
  let body = text(
    size: c.size * 1pt,
    fill: rgb(c.color).transparentize((1 - c.opacity) * 100%),
    top-edge: "bounds",
    bottom-edge: "bounds",
    eval(c.markup, mode: "markup"),
  )
  let size = measure(body)
  let origin = (left, center, right).at(int(ax * 2)) + (top, horizon, bottom).at(int(ay * 2))
  place(
    top + left,
    dx: c.x * 1pt - ax * size.width,
    dy: c.y * 1pt - ay * size.height,
    rotate(c.rotate * 1deg, origin: origin, body),
  )
}

/// The page background: the garden's SVG over the whole paper and its captions, each
/// placed by its anchor. `none` when the document has no garden.
#let background(cfg, resolve: none, ..args) = if not cfg.garden { none } else {
  context {
    let out = if resolve == none { debug(cfg, ..args) } else { debug(cfg, ..resolve(cfg)) }
    place(top + left, image(out.svg, format: "svg", width: page.width, height: page.height))
    for c in out.captions { _caption(c) }
  }
}
