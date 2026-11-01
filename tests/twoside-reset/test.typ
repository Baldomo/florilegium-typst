#import "/src/lib.typ": florilegium
#import "/src/garden.typ"

#let args = (mode: "grown", seed: 1, twoside: true)
#show: florilegium.with(..args)

// Typst's inside/outside follow the physical page, not the page counter: restarting the
// counter does not move the text block, and the garden follows the block.
#let check() = context {
  let out = garden.page-out(garden.cfg(..args))
  // the first layout pass cannot place the context yet (x = 0)
  let x = here().position().x
  if x != 0pt { assert.eq(out.mirror, x < 40mm, message: repr((here().page(), x, out.mirror))) }
}
#check()
#pagebreak()
#counter(page).update(1)
#check()
#pagebreak()
#counter(page).update(8)
#check()
