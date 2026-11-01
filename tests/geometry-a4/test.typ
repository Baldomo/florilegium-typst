#import "/src/lib.typ": florilegium
#import "/src/garden.typ"

#let args = (mode: "grown", seed: 1)
#show: florilegium.with(..args)

// svg.lua --seed 1: `wc -c` and `head -c 200`
#context {
  let out = garden.debug(garden.cfg(..args))
  assert.eq(out.svg.len(), 3076537)
  assert.eq(str(out.svg.slice(0, 200)), "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"210.00mm\" height=\"297.00mm\" viewBox=\"0 0 595.28 841.89\">\n<g transform=\"matrix(1 0 0 -1 0 841.89)\" stroke-linecap=\"round\" stroke-linejoin=\"round\">\n<clipPa")
  assert.eq(out.mirror, false)
}
