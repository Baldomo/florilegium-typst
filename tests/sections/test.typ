#import "/src/lib.typ": florilegium, bloom, theme, age, frame, textend
#import "/src/marks.typ"
#import "/src/garden.typ"
#set page(width: 21cm, height: 29.7cm)

#set heading(numbering: "1.")
#show: florilegium.with(mode: "grown", seed: 3, sections: true)
Intro.

= Title

Body #lorem(10)

=== Deep

#heading(numbering: none)[Plain]

#block[#text(size: 15.4pt, weight: "bold")[#metadata(0) <pa>#box(width: 0pt)#metadata(0) <pb>Title]]

#context {
  let ms = marks.on-page(1).filter(m => m.kind == "heading")
  let top = query(heading).first().location().position().y
  let dy = locate(<pb>).position().y - locate(<pa>).position().y
  let y = (page.height - top - dy) / 1cm
  assert.eq(ms.len(), 1)
  assert(calc.abs(ms.first().y - y) < 0.02, message: repr((ms.first().y, y)))
}
