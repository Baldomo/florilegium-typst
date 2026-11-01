#import "/src/lib.typ": florilegium
#import "/src/garden.typ"

#set page(paper: "us-letter")
#let args = (mode: "grown", seed: 2)
#show: florilegium.with(..args)


// the head rule ends where a full-width line ends
#context {
  let out = garden.debug(garden.cfg(..args))
  let x = here().position().x
  layout(size => {
    let right = x + size.width
    if x != 0pt { assert(calc.abs(right.cm() - out.geometry.textright) < 0.01, message: repr((right.cm(), out.geometry.textright))) }
  })
}
