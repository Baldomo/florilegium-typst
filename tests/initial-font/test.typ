#import "/src/lib.typ": initial, initial-font
#set page(width: 8cm, height: 9cm, margin: 1cm)
#set text(size: 11pt)

#let armenian = read("DroidSansArmenian.ttf", encoding: none)
#initial("Ա", font: armenian, lines: 3, seed: 2)[ռանց ստեղծագործական ազատության ոչինչ չի կարող ծաղկել։ Without freedom no flower grows.]

#initial-font(read("/fonts/lmroman10-bold.otf", encoding: none))
#initial("B", lines: 2)[etween two fonts, the default one and a document-level one set here.]
