// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
// Lengths, margins and the page dimensions the core lays the garden out in.

// TeX's scaled points (sp, 1/65536 TeX pt), the unit of the core's `dimens`.
#let _units = (
  // (unit, bp per unit, numerator, denominator) as in svg.lua's `units`: TeX scans
  // `<n><unit>` as n * num/den TeX points, carrying the remainder of the division.
  ("bp", 1.0, 7227, 7200),
  ("mm", 72 / 25.4, 7227, 2540),
  ("cm", 720 / 25.4, 7227, 254),
  ("in", 72.0, 7227, 100),
  ("pt", 72 / 72.27, 1, 1),
)

// svg.lua's `M.sp` (TeX's scan_dimen) for a decimal with at most five fractional digits:
// the fraction is rounded to 2^-16, then the unit's ratio is applied with the remainder
// carried. `n` is the decimal times 100000.
#let _scan-dimen(n, num, den) = {
  let neg = n < 0
  let n = calc.abs(n)
  let v = calc.div-euclid(n, 100000)
  let digits = calc.rem-euclid(n, 100000)
  let a = 0
  for k in range(5) {
    let d = calc.rem-euclid(calc.div-euclid(digits, calc.pow(10, k)), 10)
    a = calc.div-euclid(a + d * 131072, 10)
  }
  let f = calc.div-euclid(a + 1, 2)
  if num != 1 {
    let q = calc.div-euclid(v * num, den)
    let r = calc.rem-euclid(v * num, den)
    f = calc.div-euclid(num * f + 65536 * r, den)
    v = q + calc.div-euclid(f, 65536)
    f = calc.rem-euclid(f, 65536)
  }
  let sp = v * 65536 + f
  if neg { -sp } else { sp }
}

/// A length in scaled points as TeX scans it. Typst's `pt` is TeX's `bp`. A length that is
/// a short decimal (at most five digits) in bp, mm, cm, in or TeX pt (tried in this order)
/// gets exactly the integer that `\dimexpr`/svg.lua's `sp()` gives for it (`68mm` is
/// 12679822, one less than the rounded value); any other length is rounded.
/// Needs a context (`em` lengths).
#let to-sp(length) = {
  let pt = length.to-absolute().pt()
  if pt == 0 { return 0 }
  for (_, bp, num, den) in _units {
    let x = pt / bp
    let n = int(calc.round(x * 100000))
    if n != 0 and calc.abs(x - n / 100000) <= 1e-9 * calc.abs(x) {
      return _scan-dimen(n, num, den)
    }
  }
  int(calc.round(pt * 72.27 / 72 * 65536))
}

/// The margins of one page as absolute lengths `(left, right, top, bottom)`, from anything
/// `page(margin: ..)` takes: `auto` (2.5/21 of the shorter side), a length or ratio, or a
/// dictionary with `left`, `right`, `top`, `bottom`, `x`, `y`, `inside`, `outside` and
/// `rest`. `inside` is the left margin of an odd (right-hand) page; with `even: true` the
/// page is a left-hand one and inside/outside swap. Ratios are relative to the page's
/// width (left, right) or height (top, bottom).
/// Needs a context (`em` lengths).
#let resolve-margin(margin, width, height, even: false) = {
  let side(v, of) = if v == auto {
    2.5 / 21 * calc.min(width, height)
  } else if type(v) == ratio {
    v * of
  } else if type(v) == relative {
    v.length.to-absolute() + v.ratio * of
  } else {
    v.to-absolute()
  }
  let m = if type(margin) == dictionary { margin } else { (rest: margin) }
  let get(..keys) = {
    for k in keys.pos() {
      if k in m and m.at(k) != none { return m.at(k) }
    }
    m.at("rest", default: auto)
  }
  let (inner, outer) = if even { ("outside", "inside") } else { ("inside", "outside") }
  (
    left: side(get("left", inner, "x"), width),
    right: side(get("right", outer, "x"), width),
    top: side(get("top", "y"), height),
    bottom: side(get("bottom", "y"), height),
  )
}

/// The `dimens` the plugin's `page` takes (integers in sp, as `\geometry` leaves them),
/// from the paper size, the margins of this page as `resolve-margin` returns them, and
/// the head and foot distances. The text block starts at `left`; `topmargin` is what is
/// left of the top margin above the header (svg.lua's `from_margins`). Both side margins
/// get the same value: the Typst layer reports the margins as laid out on this very page.
#let dimens(page-width, page-height, margins, headheight, headsep, footskip) = {
  let sp = to-sp
  let inch = 4736286
  let (pw, ph) = (sp(page-width), sp(page-height))
  let left = sp(margins.left)
  let tw = pw - left - sp(margins.right)
  let th = ph - sp(margins.top) - sp(margins.bottom)
  (
    paperwidth: pw,
    paperheight: ph,
    textwidth: tw,
    textheight: th,
    oddsidemargin: left - inch,
    evensidemargin: left - inch,
    topmargin: sp(margins.top) - inch - sp(headheight) - sp(headsep),
    headheight: sp(headheight),
    headsep: sp(headsep),
    footskip: sp(footskip),
    hoffset: 0,
  )
}
