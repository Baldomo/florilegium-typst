#import "/src/lib.typ": florilegium, bloom, theme, age, frame, textend
#import "/src/marks.typ"
#import "/src/garden.typ"
#set page(width: 21cm, height: 29.7cm)

#let cfg = garden.cfg(mode: "grown", seed: 3)
#show: florilegium.with(mode: "grown", seed: 3)
One
#context {
  let a = marks.page-args(cfg)
  assert.eq((a.theme, a.age, a.marks, a.spilly), ("flora", none, (), none))
}
#pagebreak()
#theme("geometric") #age(0.9) Two #bloom() here
#context {
  let a = marks.page-args(cfg)
  assert.eq((a.theme, a.age, a.spilly), ("geometric", 0.9, none))
  assert.eq(a.marks.map(m => (m.kind, m.arg)), (("bloom", ""),))
}
#pagebreak()
Three
#context {
  let a = marks.page-args(cfg)
  assert.eq((a.theme, a.age, a.marks), ("geometric", 0.9, ()))
}
