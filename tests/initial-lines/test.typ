#import "/src/lib.typ": initial
#set page(width: 12cm, height: 8cm, margin: 1cm)
#set text(size: 11pt)
#set par(justify: false)

// A tag at a line start is hoisted to the previous line's end, so each line starts with
// an "x" and the tag follows it.
#let line(i) = [x#metadata(i)#label("m" + str(i))]

#initial("A")[#line(1) \ #line(2) \ #line(3) \ #line(4) \ #line(5) \ #line(6)]

#context {
  let w = measure[x].width
  let x(i) = query(label("m" + str(i))).last().location().position().x
  for i in range(1, 5) { assert(x(i) > 1cm + w + 1pt, message: repr(x(i))) }
  assert(calc.abs(x(5) - 1cm - w) < 0.01pt, message: repr(x(5)))
}
