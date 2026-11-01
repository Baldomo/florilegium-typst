#import "/src/lib.typ": florilegium, ornament
#set page(width: 10cm, height: 4cm, margin: 1cm)
#set text(size: 12pt)

Text #ornament("flower") and #ornament("bee", size: 2em, angle: 20deg, ink: red) x #ornament("leaf", seed: 4) y
#ornament("sparkle", accent: orange, seed: 4) #ornament("leaf", seed: 4)

#context {
  let b = measure(ornament("flower", size: 20pt))
  assert.eq(b.width, 20pt)
  assert.eq(b.height, 17pt)
}
#show: florilegium.with(garden: false)
#ornament("hedera", size: 24pt)
