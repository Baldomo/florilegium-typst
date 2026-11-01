#import "/src/lib.typ": florilegium, draw
#set page(width: 8cm, height: 6cm, margin: 0pt)
#show: florilegium.with(theme: "math", garden: false, seed: 2, margin: 0pt)


#place(top + left, dx: 2cm, dy: 3cm, draw("flower", 1.5cm, seed: 3))

#place(top + left, dx: 5cm, dy: 3cm, draw("knot", 2cm, seed: 5))

#place(top + left, dx: 5cm, dy: 1cm, draw("leaf", 1cm, ink: red))

#place(top + left, dx: 0.1cm, dy: 0.1cm, square(size: 2pt, fill: black))
