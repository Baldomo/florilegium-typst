#import "/src/lib.typ": initial, initial-font
#set page(width: 8cm, height: 9cm, margin: 1cm)
#set text(size: 11pt)

#let armenian = read("DroidSansArmenian.ttf", encoding: none)
#initial("Ա", font: armenian, lines: 3, seed: 2)[ռանց ստեղծագործական ազատության ոչինչ չի կարող ծաղկել։ Without freedom no flower grows.]

#initial-font(armenian)
#initial("Բ", lines: 2)[ռանց ծաղիկ, a document-level font set here, drawn with the same letter shapes.]
