#import "/src/lib.typ": initial, _initial-tile
#set page(width: 12cm, height: 10cm, margin: 1cm)
#set text(size: 11pt)
#set par(justify: false)

// The tile's top is the first line's cap top, its bottom the baseline of line `lines`,
// measured from plain lines of the same paragraph style.
#let line(i) = [x#metadata(i)#label("p" + str(i))]
#line(1) \ #line(2) \ #line(3)

#for n in (1, 2, 3) [
  #initial("A", lines: n)[#line(10 * n) \ #line(10 * n + 1) \ #line(10 * n + 2) \ #line(10 * n + 3)]
]

#context {
  let y(i) = query(label("p" + str(i))).last().location().position().y
  let pitch = y(2) - y(1)
  let cap = measure(text("H")).height
  let tiles = query(<florilegium-initial-tile>)
  assert.eq(tiles.len(), 3)
  for (k, t) in tiles.enumerate() {
    let n = k + 1
    let top = t.location().position().y
    let first = y(10 * n)
    assert(calc.abs(top - (first - cap)) < 0.5pt, message: repr((n, top, first, cap)))
    let h = measure(_initial-tile("A", lines: n)).height
    let last = y(10 * n + n - 1)
    assert(calc.abs(top + h - last) < 0.5pt, message: repr((n, top, h, last)))
    assert(calc.abs(last - first - (n - 1) * pitch) < 0.01pt)
  }
}
