#import "/src/lib.typ": florilegium, bloom, theme, age, frame, textend
#import "/src/marks.typ"
#import "/src/garden.typ"
#set page(width: 21cm, height: 29.7cm)

#set heading(numbering: "1.")
#show: florilegium.with(mode: "grown", seed: 3, sections: true)
Intro.

= Title

#show heading.where(level: 2): set text(size: 20pt)
== Sub

Body #lorem(10)

=== Deep

#heading(numbering: none)[Plain]

#let oracle(size, a, b) = block[#text(size: size, weight: "bold")[#metadata(0) #label(a)#box(width: 0pt)#metadata(0) #label(b)Title]]
#oracle(15.4pt, "pa1", "pb1")
#oracle(20pt, "pa2", "pb2")

#context {
  let ms = marks.on-page(1).filter(m => m.kind == "heading")
  let hs = query(heading).filter(h => h.numbering != none and h.level <= 2)
  assert.eq(ms.map(m => m.arg), (1, 2))
  assert.eq(hs.len(), 2)
  for (i, h) in hs.enumerate() {
    let dy = locate(label("pb" + str(i + 1))).position().y - locate(label("pa" + str(i + 1))).position().y
    let y = (page.height - h.location().position().y - dy) / 1cm
    assert(calc.abs(ms.at(i).y - y) < 0.02, message: repr((i, ms.at(i).y, y)))
  }
}
