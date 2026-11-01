#import "/src/lib.typ": florilegium, bloom, theme, age, frame, textend
#import "/src/marks.typ"
#import "/src/garden.typ"
#set page(width: 21cm, height: 29.7cm)

#show: florilegium.with(mode: "grown", seed: 3)
One
#pagebreak()
#theme("geometric") Two
#pagebreak()
Three
#context {
  let cfg = garden.cfg(mode: "grown", seed: 3)
  assert.eq(marks.effective(1, cfg).theme, "flora")
  assert.eq(marks.effective(2, cfg).theme, "geometric")
  assert.eq(marks.effective(3, cfg).theme, "geometric")
}
