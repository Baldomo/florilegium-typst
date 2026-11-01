// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
// Procedurally generated Delft-blue floral page margins (a port of florilegium).

#import "garden.typ" as _garden
#import "marks.typ" as _marks
#import "core.typ" as _core
#import "geometry.typ" as _geometry
#import "@preview/droplet:0.3.1": dropcap

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
    background: _garden.background(cfg, resolve: _marks.page-args),
  )
  show heading: it => {
    if sections and it.numbering != none and it.level <= 2 {
      // Under the heading's own text style, a heading probe would be styled twice.
      let dy = measure(text[H]).height
      _marks.heading-mark(it.level, dy, it.location())
    }
    it
  }
  _garden.cfg-state.update(cfg)
  body
}

#let bloom = _marks.bloom
#let theme = _marks.theme
#let age = _marks.age
#let frame = _marks.frame
#let textend = _marks.textend

#let _ornaments = counter("florilegium-ornament")
#let _draws = counter("florilegium-draw")

// The template's settings at this location (none outside the template) and the theme in
// effect on this page. Needs a context.
#let _setting(ink, accent, theme) = {
  let cfg = _garden.cfg-state.get()
  let hex(c) = _garden._hex(c)
  let pick(v, key) = if v != none { hex(v) } else if cfg != none { hex(cfg.at(key)) } else { "" }
  (
    ink: pick(ink, "ink"), accent: pick(accent, "accent"),
    theme: if theme != none { theme } else if cfg != none { _marks.effective(here().page(), cfg).theme } else { none },
  )
}

#let _svg(r, dx, dy) = if r.width == 0 or r.height == 0 { none } else {
  place(top + left, dx: dx - r.dx * 1pt, dy: dy - r.dy * 1pt,
    image(r.svg, format: "svg", width: r.width * 1pt, height: r.height * 1pt))
}

/// One primitive in a box of side `size` on the baseline, like a letter (depth .15 size).
///
/// - name: a primitive of the theme in effect (flora's: leaf, flower, bee, ..).
/// - seed: auto (the k-th ornament of the document always looks the same) or an integer;
///   "random" is not supported: Typst documents are deterministic.
#let ornament(name, size: 1em, ink: none, accent: none, angle: 0deg, seed: auto) = {
  if seed == "random" {
    panic("florilegium: seed: \"random\" is not supported in Typst; the build is deterministic, give an integer seed")
  }
  _ornaments.step()
  context {
    let s = measure(box(width: size)).width
    let st = _setting(ink, accent, none)
    let r = _core.ornament((
      name: name, size: _geometry.to-sp(s), angle: angle.deg(),
      count: _ornaments.get().first(), seed: if seed == auto { none } else { seed },
      ink: st.ink, accent: st.accent, theme: st.theme,
    ))
    box(width: s, height: .85 * s, baseline: .15 * s, _svg(r, 0pt, .85 * s))
  }
}

/// A primitive drawn at the current point: a zero-size box whose position is the
/// primitive's origin. `size` is its length; `opts` are the primitive's options (tone, w,
/// bend, side, depth, ..). Without `seed` the drawing is determined by the template seed
/// (1 if none), the physical page and the count of draws: seed * 7919 + page * 131 + k * 977.
#let draw(name, size, ink: none, accent: none, theme: none, seed: auto, ..opts) = {
  _draws.step()
  context {
    let st = _setting(ink, accent, theme)
    let cfg = _garden.cfg-state.get()
    let seed = if seed != auto { seed } else {
      let base = if cfg != none and cfg.seed != none { cfg.seed } else { 1 }
      base * 7919 + here().page() * 131 + _draws.get().first() * 977
    }
    let r = _core.draw((
      name: name, size: size.to-absolute() / 1cm, opts: opts.named(), seed: seed,
      ink: st.ink, accent: st.accent, theme: st.theme,
    ))
    box(width: 0pt, height: 0pt, _svg(r, 0pt, 0pt))
  }
}

#let _initials = counter("florilegium-initial")
#let _initial-font = state("florilegium-initial-font", auto)
#let _bundled-font = read("../fonts/lmroman10-bold.otf", encoding: none)

/// Sets the initials' font for the rest of the document: the bytes of an OpenType/TrueType
/// file, e.g. `read("my.otf", encoding: none)`, or `auto` for the bundled Latin Modern
/// Roman Bold. Overrides the template's `initial-font`.
#let initial-font(font) = _initial-font.update(font)

// The tile of an initial: a box of side `lines` baselines (`S`), the letter drawn in it,
// and the letter itself invisibly over it, so that it can be copied.
#let _initial-tile(letter, lines: 4, font: auto, seed: auto, ink: none, accent: none) = {
  assert(type(letter) == str and letter.clusters().len() == 1,
    message: "florilegium: the initial's letter must be a single character, got " + repr(letter))
  if seed == auto { _initials.step() }
  context {
    let cap = measure(text("H")).height
    let side = (lines - 1) * (cap + par.leading.to-absolute()) + cap
    let cfg = _garden.cfg-state.get()
    let font = if font != auto { font } else if _initial-font.get() != auto { _initial-font.get() } else if cfg != none and cfg.initial-font != auto { cfg.initial-font } else { _bundled-font }
    let st = _setting(ink, accent, none)
    let r = _core.initial((
      letter: letter, side: _geometry.to-sp(side), font: font,
      seed: if seed == auto { _initials.get().first() } else { seed } * 7919 + 5003,
      ink: st.ink, accent: st.accent,
    ))
    box(width: side, height: side, {
      [#metadata(none)<florilegium-initial-tile>]
      _svg(r, 0pt, side)
      place(top + left, text(fill: rgb(0, 0, 0, 0), size: side, letter))
    })
  }
}

/// A paragraph with a decorated initial: a procedurally drawn tile of `lines` lines in
/// the Delft-blue style, holding `letter`, with `body` (the rest of the paragraph) beside it.
///
/// - lines: the height of the initial in lines of the paragraph.
/// - font: `auto` (the document's `initial-font`, else Latin Modern Roman Bold) or the
///   bytes of a font file.
/// - seed: auto (the k-th initial of the document always looks the same) or an integer.
#let initial(lines: 4, font: auto, seed: auto, ink: none, accent: none, letter, body) = {
  dropcap(
    height: lines, gap: 0.6em,
    _initial-tile(letter, lines: lines, font: font, seed: seed, ink: ink, accent: accent),
    body,
  )
}
