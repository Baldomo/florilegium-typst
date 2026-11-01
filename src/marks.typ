// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
// Marks: positions in the text that steer the garden (headings, blooms, theme, age,
// animation frames, the end of the text), the counterpart of florilegium.mark.

#let _mark(kind, arg, ..extra) = [#metadata((kind: kind, arg: arg, ..extra.named()))<florilegium-mark>]

/// A bloom beside this line (inline: y is the line's baseline); `motif` "" lets the garden choose.
#let bloom(motif: "") = [#h(0pt)#_mark("bloom", motif)]

/// The theme from this page on ("flora", "geometric", "math"..).
#let theme(name) = [#parbreak()#_mark("theme", name)]

/// The season from this page on: 0 spring, 0.5 summer, 1 autumn (ignored with `aging: true`).
#let age(v) = [#parbreak()#_mark("age", v)]

/// Show frame `k` of `n` of the garden's growth from this page on.
#let frame(k, n, explain: false) = [#parbreak()#_mark("frame", (k: k, n: n, explain: explain))]

/// Where the text ends: the garden's spill flows under it (grown mode).
#let textend() = [#parbreak()#_mark("textend", none)]

/// The heading mark, placed before a heading: the heading's top (`loc`, after the spacing
/// above it) plus `dy`, the first line's height.
#let heading-mark(level, dy, loc) = _mark("heading", level, dy: dy, loc: loc)

/// The marks on physical page `p`, in document order, with `y` in cm from the page's
/// bottom edge. Needs a context.
#let on-page(p) = {
  let h = page.height
  query(<florilegium-mark>)
    .map(m => (m.value, m.value.at("loc", default: m.location())))
    .filter(((v, loc)) => loc.page() == p)
    .map(((v, loc)) => {
      let dy = v.at("dy", default: 0pt)
      (kind: v.kind, arg: v.arg, y: (h - loc.position().y - dy) / 1cm)
    })
}

/// What applies to physical page `p`: the plugin's `theme`, `age`, `frame` and `spilly`.
/// theme, age and frame are the last such mark on or before the page; with `aging` the age
/// follows the logical page number and the last page (as `auto_age` does).
#let effective(p, cfg, n: auto) = {
  let theme = cfg.theme
  let age = none
  let frame = none
  for m in query(<florilegium-mark>) {
    if m.location().page() > p { break }
    let v = m.value
    if v.kind == "theme" { theme = v.arg } else if v.kind == "age" { age = v.arg } else if v.kind == "frame" { frame = v.arg }
  }
  if cfg.aging {
    let last = counter(page).final().first()
    let n = if n == auto { counter(page).get().first() } else { n }
    age = if last > 1 { calc.min(1, (n - 1) / (last - 1)) } else { 0.5 }
  }
  let spilly = on-page(p).filter(m => m.kind == "textend").map(m => m.y).last(default: none)
  (theme: theme, age: age, frame: frame, spilly: spilly)
}

/// The arguments `garden.background` takes for the page being laid out. Needs a context.
#let page-args(cfg) = {
  let p = here().page()
  let e = effective(p, cfg)
  let ms = on-page(p).filter(m => m.kind in ("heading", "bloom"))
  (marks: ms, theme: e.theme, age: e.age, frame: e.frame, spilly: e.spilly)
}
