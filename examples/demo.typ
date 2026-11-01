// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
// The authored composition: the text of florilegium.tex.
// From lib/:  typst compile examples/demo.typ --root .
// Installed:  replace the import by  #import "@preview/florilegium:0.1.0": *
#import "../src/lib.typ": *

#set page(paper: "a4")
#set text(size: 11pt)
#set par(justify: true)
#show: florilegium.with()
#set page(
  header: align(right, text(size: 9pt, smallcaps[3#h(1em)Procedural ornament])),
  footer: align(right, text(size: 9pt, context counter(page).display())),
)

#let env(kind, title: none, italic: false, body) = block(above: 1em, below: 1em, {
  strong[#kind#if title != none [ (#title)].]
  [ ]
  if italic { emph(body) } else { body }
})
#let construction(n, title: none, body) = env("Construction 3." + str(n), title: title, body)
#let proof(title, body) = block(above: 1em, below: 1em, { emph[Proof #title.] + [ ] + body + h(1fr) + sym.square })

#heading(level: 2, numbering: none)[3.2#h(1em)The garden in the margin]

Everything in the left margin of this page is drawn while the page is being shipped out,
from a few hundred lines of Lua run by LuaTeX. There are no image files, no imported
paths and no external programs; every petal is a parametric curve evaluated for this page
and written straight into it as PDF path operators. This section records how it is done,
so that the margin documents itself.

#env("Definition 3.1")[
  A _primitive_ is a closed outline $gamma: [0,1] -> bb(R)^2$ authored by hand
  together with a small set of free parameters $(p_1, dots, p_k)$ and a recipe for painting
  it. An _instance_ of the primitive samples the parameters from fixed intervals and
  paints $gamma$ in a local frame chosen by the caller. The composition on this page is
  _authored_; only the parameters are _sampled_. This is what "procedural" means
  here: no flower is grown from a grammar, but no two petals share a shape.
]

#construction(2, title: "Palette")[
  There is one document color, `ink`, and from it a ramp $t |-> op("tone")(t)$
  from the darkest navy at $t=0$ to a pale mist at $t=1$. The ramp is interpolated in
  OKLCH between six stops whose lightness runs from $0.25$ to $0.92$ and whose chroma peaks
  at $0.175$ for the cobalt at $t=0.38$; the hue is that of `ink`, drifting $6 degree$
  toward violet in the darks and $8 degree$ toward cyan in the lights, as ultramarine does when
  it is thinned with water, and the chroma scales with that of `ink`. Mixing the
  document color with white, as `ink!22` would, desaturates toward gray instead;
  the perceptual ramp keeps the light end blue. A second color, `accent`, is a copper
  used only for hairlines.
]

#construction(3, title: "Petal")[
  A petal of length $1$ pointing along $+y$ has a spine $x = b v^2$, $y = v$ ($b$ bends the
  tip) and the obovate half-width
  $ w(v) = cases(
    W(0.16 + 0.84 sin^(1.3) (pi v)/1.24)\, & v < 0.62,
    W sqrt(1 - q^2)\, quad q = (v - 0.62)/0.38\, & v >= 0.62,
  ) quad W in [0.40, 0.48], $
  a narrow claw, widest at $0.62$, with an elliptical cap. The margin is moved along the
  normal by $0.03 v n(6v)$, where $n$ is a three-octave gradient noise seeded per petal, so
  it wavers more toward the tip; four petals in ten also get a shallow notch. The outline is
  sampled once and used twice: as a clipping path, and as the stroke that gives the rim.
  Inside the clip the petal is painted like a watercolor, in the order a painter would: a
  flat wash at $op("tone")(t_0)$, $t_0 in [0.30, 0.46]$; eight stacked white blobs at
  the base, each at $10%$ opacity, for the pale heart; fifteen irregular blotches
  multiplied over the wash, most darker, some lighter, and a dozen dark specks for the
  granulation; five to eight darker brush strokes running out from the base; the outline
  stroked $1.6 ell$ bp wide for a petal $ell$ cm long and multiplied, of which only the inner half shows, for the pigment
  that pools at the edge of a wash; nine to thirteen white streaks lifted out in a fan from
  the base; and a few fine dark veins. There is no outline in the usual sense.
]

#construction(4, title: "Tapered strokes")[
  Every streak, vein and rib is a filled shape rather than a stroke, so that it tapers. For a
  cubic Bézier with control points $P_0, dots, P_3$ and widths $w_0$, $w_1$ at its ends, push
  each $P_i$ along its normal by $plus.minus 1/2 w(i\/3)$, with the normals at $P_1$ and $P_2$ taken
  from the chords $P_2 - P_0$ and $P_3 - P_1$. The two offset curves joined at the ends are a
  tapered stroke in two Bézier segments, five times smaller in the file than a sampled
  ribbon and indistinguishable from one for a gentle curve.
]

#construction(5, title: "Flowers")[
  A _flower_ is five petals at $72 degree$ intervals with $plus.minus 6 degree$ of jitter and lengths
  in $[0.9, 1.06]$, then a heart of ten white rays, a white disc, fourteen stamens and a copper
  eye. A quarter of the flowers are painted a shade deeper. A _lily_ is six pointed
  petals, $w(v) = 2W sin^(0.85)(pi v^(0.8))(1 - 0.3v)$, arranged in a star and then squashed
  vertically by a factor in $[0.45, 0.62]$: the star is seen at a tilt, so the petals that point
  away are short, and since they are drawn first and darker they read as the back of the
  flower. A _blossom_ is the same idea at a few millimeters, and a _floret_, the
  smallest, is five petals filled as one shape around a dark eye; below $0.7$ mm in radius it is a
  ten-pointed star. Buds, bellflowers and sepals are blades on the same spine.
]

#construction(6, title: "Leaves")[
  A leaf of length $1$ has the envelope $e(v) = 2.3 W v^(0.55) (1 - v)^(0.95)$. An entire leaf is
  that envelope with fine teeth. A lobed leaf, like acanthus, is a narrow body of half-width
  $(1 - 0.75 d) e(v)$, where $d$ is the depth of the cut, together with three to five pointed
  lobes on each side; each lobe is itself a blade that leaves the midrib at
  $38 degree$--$58 degree$ toward the tip and reaches the envelope. Body and lobes are filled as
  one shape, under the nonzero rule, so there is no seam. One half of the leaf is multiplied
  darker, as if it were folded along its rib; then come the blotches, the pooled edge, and a
  white midrib with a white vein along every lobe.
]

#construction(7, title: "Sprays")[
  A _cyme_, the inflorescence of the forget-me-not, is a stem whose heading turns by
  $plus.minus tau (s\/L)^(2.3)$ at arc length $s$, with $tau in [170 degree, 270 degree]$: straight at first,
  it curls like a scorpion's tail at the end. Eleven to seventeen florets crowd along it,
  shrinking into buds toward the curl, and a second, shorter curl forks off it three times in
  five. A _sprig_ is a stem that forks two or three ways four times, each fork $0.58$ to
  $0.74$ the length of its parent, with florets and buds at the tips. An _umbel_ fans nine
  to thirteen rays over $110 degree$, each ending in a small umbel of starry florets.
]

#text(weight: "bold")[3.2.1#h(1em)Composition]

The garden is a plant, not an arrangement. It grows from one root in the bottom-left
corner of the page, where the base of the stems curls into a scroll. Three stems run up the
column through hand-picked control points, interpolated by Catmull--Rom splines and jittered
by a millimeter or so on each page: the tallest ends in the large flower at the top, one in
a lily, one in a flower half-way up. A fourth stem leaves the root along the bottom of the
page, carrying small leaves that thin out toward its end, and runs out into a copper
hairline; it is the footer. The header is a printer's ivy leaf in copper whose stem becomes
the head rule.

Leaves hang at chosen fractions of the stems' length, alternately left and right. Every
other motif sits at a chosen point and is joined to a chosen stem by a _branch_: a cubic
Bézier that leaves the stem along the stem's own direction turned $28 degree$ to $48 degree$
toward the target, and arrives along the motif's axis. Because a branch starts tangent to
its parent and ends tangent to its flower, every flower can be traced back to the root, and
that, more than any detail of the painting, is what makes the margin look grown. Behind
everything are one or two _ghosts_: flowers painted with every tone lifted toward the
paper, $t |-> t + 0.62(1 - t)$, and without their fine detail, which give the column depth.

#env("Remark 3.8", title: "Procedural is not random")[
  The generator is seeded from the page number before the garden is drawn. Page $n$
  therefore receives the same garden on every compilation, and a different one from page
  $n+1$. The variation the reader notices between pages is real; the reproducibility is what
  makes the document stable enough to typeset.
]

#env("Proposition 3.9", italic: true)[
  Let $P$ be the number of primitive instances on a page and $S$ the mean number of path
  points per instance. The cost of a page is $Theta(P S)$ floating-point operations in Lua and
  $Theta(P S)$ bytes of output, and TeX does no arithmetic at all; in practice a page takes
  well under a second, for $P approx 400$ and $S approx 100$.
]

#proof("sketch")[
  Every coordinate is computed once in Lua, transformed to the page, and printed with one
  decimal of a PDF point into a single literal node, which TeX only carries to the page.
  The same curves can be computed in TeX itself with `pgfmath`, but it does
  arithmetic by macro expansion in TeX dimensions, and a page then takes about a minute. The
  size of the output is the real constraint: about
  three megabytes of path operators per page, which compress to under one.
]

#env("Exercise 3.10")[
  The composition above is authored. Replace it by a growth process in the spirit of Wong,
  Zongker and Salesin (_Computer-generated floral ornament_, SIGGRAPH 1998): starting
  from the stems, repeatedly find the largest free disc in the margin that is within reach of
  a stem, choose a primitive that fits it, and grow a branch to it, until no disc larger than
  a threshold remains. The ornament then adapts to any text block, and a page whose text
  ends early grows a garden into the space below it.
]

#text(weight: "bold")[3.2.2#h(1em)Building and sharing]

The package is `florilegium.sty` with nine Lua files beside it: `ink.lua`
(geometry, color and PDF output), `flora.lua` (the primitives),
`authored.lua` and `grow.lua` (the two compositions), `initial.lua`
(the decorated initials), two themes and the figures one of them draws from
(`theme-geometric.lua`: the same plant cut from flat shapes;
`theme-math.lua`: figures, symbols and equations cascading down the margin;
`figures.lua`) and `florilegium.lua` (the glue to TeX). It needs LuaLaTeX and one compilation, at
about a second per page. To share it, the three routes that work are, in increasing order
of effort: paste the files into a blank Overleaf project with the compiler set to
LuaLaTeX and share the link; put them in a Git repository with a continuous integration
job that compiles the PDFs on every push and attaches them to tagged releases; or publish
the package so that any document can `\usepackage` the garden and
choose its own seed, column and hue.

#env("Remark 3.11")[
  Two mistakes cost time and are worth recording. A literal written into the page bypasses
  TikZ, so it also bypasses TikZ's transformations: a primitive called inside
  `\begin{scope}[shift=...,rotate=...]` must read pgf's current matrix itself (it is in
  `\pgf@pt@aa` to `\pgf@pt@bb` and the dimensions `\pgf@pt@x`,
  `\pgf@pt@y`) and apply it in Lua. And a Lua function that returns two values, placed
  in the middle of a table constructor such as `{x0, y0, f(t), x1, y1}`, contributes only
  its first value; the rest of the table shifts, and the error surfaces three functions
  later as arithmetic on `nil`.
]
