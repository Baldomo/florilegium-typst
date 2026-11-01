#import "/src/lib.typ": florilegium, bloom, theme, age, frame, textend
#import "/src/marks.typ"
#import "/src/garden.typ"
#set page(width: 21cm, height: 29.7cm)

#show: florilegium.with(mode: "grown", seed: 3)
#lorem(400)

#textend()
#metadata(0) <end>

#context {
  let p = here().page()
  let e = marks.effective(counter(page).final().first(), garden.cfg(mode: "grown", seed: 3))
  let ms = marks.on-page(query(<florilegium-mark>).last().location().page())
  let y = (page.height - locate(<end>).position().y) / 1cm
  assert(calc.abs(e.spilly - y) < 0.02, message: repr((e.spilly, y)))
  assert(e.spilly > 0 and e.spilly < page.height / 1cm)
  assert(calc.abs(ms.last().y - y) < 0.02)
  let cfg = garden.cfg(mode: "grown", seed: 3)
  assert(garden.page-out(cfg, spilly: e.spilly).svg.len() != garden.page-out(cfg).svg.len())
}
