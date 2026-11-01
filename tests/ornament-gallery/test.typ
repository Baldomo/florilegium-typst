#import "/src/lib.typ": ornament
#set page(width: 16cm, height: 8cm, margin: 1cm)
#set text(size: 12pt)
#let names = ("leaf", "leaflet", "petal", "bud", "bell", "sprig", "cyme", "umbel", "tendril", "flower", "blossom", "floret", "lily", "seedhead", "butterfly", "bee", "snail", "bird", "sparkle", "rosette", "hedera")
#for n in names [#ornament(n, size: 2em) ]
