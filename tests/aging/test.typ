#import "/src/lib.typ": florilegium, bloom, theme, age, frame, textend
#import "/src/marks.typ"
#import "/src/garden.typ"
#set page(width: 21cm, height: 29.7cm)

#show: florilegium.with(mode: "grown", seed: 3, aging: true)
One
#pagebreak()
#age(0.9) Two
#pagebreak()
Three
#context {
  let cfg = garden.cfg(mode: "grown", seed: 3, aging: true)
  assert.eq((1, 2, 3).map(p => marks.effective(p, cfg, n: p).age), (0, 0.5, 1))
}
