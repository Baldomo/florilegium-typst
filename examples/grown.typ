// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
// The grown composition with flowers at the headings, blooms, an initial,
// ornaments in the text, aging seasons, and a last page in the math theme.
// From lib/:  typst compile examples/grown.typ --root .
// Installed:  replace the import by  #import "@preview/florilegium:0.1.0": *
#import "../src/lib.typ": *

#set page(paper: "a4")
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.1")
#show: florilegium.with(mode: "grown", sections: true, aging: true)
#set page(
  header: align(right, text(size: 9pt, smallcaps[A year in the margin])),
  footer: align(right, text(size: 9pt, context counter(page).display())),
)

= Spring

#initial("E", lines: 5)[very page of this document grows its own garden, but the
gardens here also read the text they surround. This first page is spring: the plant is
young, its leaves are small and pale, and it is mostly in bud; the last page is winter
in a different style. The sections that follow show what the options do, along with a
few things that need no option at all. #ornament("leaf", size: 1.2em) #ornament("flower", size: 1.4em)]

The letter that opens the first paragraph was set with `initial("E", lines: 5)[very ...]`.
It is drawn like a petal, with a vine crossing it, in a tile five lines tall; the paragraph
still copies and searches as _Every_.

== Flowers at the headings

With `sections: true`, every numbered section and subsection gets a flower in the margin,
beside its heading: a large one for a section, a spray of small ones for a subsection,
painted in the copper of the accents. Where the margin is already taken, the flower is
made smaller or left out.

== A flower where the text asks for one

`bloom()` puts a flower in the margin beside the line in which it stands, like this
one#bloom(), and `bloom(motif: "lily")` asks for a particular motif#bloom(motif: "lily"):
lily, blossoms, cyme, umbel, bud, or a butterfly. It costs nothing on pages that have no
marks and does not move the text.

= Summer

By the middle of the document the garden is in full bloom: open flowers, star lilies,
forget-me-nots curling at their tips, umbels. The seasons change what grows, not where.
In spring sprays of closed buds take the place of most open flowers; summer is the garden
as it always was.

== Visitors

Most pages have visitors: one or two butterflies and bees in the open spaces, a snail
crawling up one of the main stems, and on about one page in five a bird perched on a
twig of its own. They are drawn from their own random stream, so a page without them is
exactly the page it would have been, and `visitors: false` leaves them all out.

#lorem(120)

#lorem(120)

= Autumn

#lorem(150)

#bloom(motif: "umbel") #lorem(60)

#lorem(150)

#pagebreak()
#theme("math")

= Mathematics

The margin can also cascade figures, symbols and equations: `#theme("math")` switches
the theme from this page on, and `theme("flora")` would switch it back. #lorem(60)

#textend()
