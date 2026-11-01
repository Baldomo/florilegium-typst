#import "/src/lib.typ": florilegium
#import "/src/garden.typ"


#let args = (mode: "grown", seed: 2, margin: auto)
#show: florilegium.with(..args)
#set page(margin: (left: 50mm, right: 15mm, rest: 25mm))

// the head rule ends where a full-width line ends
#context {
  let out = garden.page-out(garden.cfg(..args))
  let x = here().position().x
  layout(size => {
    let right = x + size.width
    if x != 0pt { assert(calc.abs(right.cm() - out.geometry.textright) < 0.01, message: repr((right.cm(), out.geometry.textright))) }
  })
}
