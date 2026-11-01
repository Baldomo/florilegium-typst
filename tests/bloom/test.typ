#import "/src/lib.typ": florilegium, bloom, theme, age, frame, textend
#import "/src/marks.typ"
#import "/src/garden.typ"
#set page(width: 21cm, height: 29.7cm)

#show: florilegium.with(mode: "grown", seed: 3)
First line.

#bloom() Here
#metadata(0) <probe> after.

#context {
  let m = marks.on-page(1).filter(m => m.kind == "bloom").first()
  let h = page.height
  let y = (h - locate(<probe>).position().y) / 1cm
  assert(calc.abs(m.y - y) < 0.02, message: repr((m.y, y)))
  let cfg = garden.cfg(mode: "grown", seed: 3)
  let a = garden.debug(cfg, marks: (m,))
  let b = garden.debug(cfg)
  assert(a.svg.len() != b.svg.len())
}

#context {
  assert.eq(measure[x#bloom(),].width, measure[x,].width)
  assert.eq(measure[x#bloom(motif: "lily"),].width, measure[x,].width)
  assert.eq(measure[x#metadata(0)<florilegium-mark>,].width, measure[x,].width)
  assert(query(<florilegium-mark>).len() >= 1)
}
