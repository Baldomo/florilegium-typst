#import "/src/lib.typ": florilegium, bloom, theme, age, frame, textend
#import "/src/marks.typ"
#import "/src/garden.typ"
#set page(width: 21cm, height: 29.7cm)

#show: florilegium.with(mode: "grown", seed: 3)
One
#context {
  let cfg = garden.cfg(mode: "grown", seed: 3)
  let plain = garden.debug(cfg).svg
  let f(k) = garden.debug(cfg, frame: (k: k, n: 3, explain: false)).svg
  assert.eq(f(3), plain)
  assert(f(1) != plain)
  assert(f(2) != plain)
}
