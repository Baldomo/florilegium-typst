#import "/src/lib.typ": florilegium
#import "/src/garden.typ"

#let args = (mode: "grown", seed: 1, twoside: true)
#show: florilegium.with(..args)

// the garden is mirrored exactly where the left margin is the narrower one
#let check() = context {
  let out = garden.debug(garden.cfg(..args))
  // the first layout pass cannot place the context yet (x = 0)
  let x = here().position().x
  if x != 0pt { assert.eq(out.mirror, x < 40mm, message: repr((here().page(), x, out.mirror))) }
}
#check()
#pagebreak()
#check()
#pagebreak()
#check()
