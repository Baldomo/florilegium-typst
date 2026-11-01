#import "/src/lib.typ": florilegium, bloom, theme, age, frame, textend
#import "/src/marks.typ"
#import "/src/garden.typ"
#set page(width: 21cm, height: 29.7cm)

#show: florilegium.with(mode: "grown", seed: 3)
#lorem(400)

#textend()

#context {
  let p = here().page()
  let e = marks.effective(counter(page).final().first(), garden.cfg(mode: "grown", seed: 3))
  let ms = marks.on-page(query(<florilegium-mark>).last().location().page())
  assert.eq(e.spilly, ms.last().y)
  let cfg = garden.cfg(mode: "grown", seed: 3)
  assert(garden.debug(cfg, spilly: e.spilly).svg.len() != garden.debug(cfg).svg.len())
}
