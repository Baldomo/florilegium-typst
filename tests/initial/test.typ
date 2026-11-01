#import "/src/lib.typ": florilegium, initial, _initial-tile
#set page(width: 12cm, height: 7cm, margin: 1cm)
#set text(size: 11pt)
#set par(justify: true)

#initial("L")[orem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.]

#initial("D", ink: red, accent: orange, seed: 3, lines: 3)[uis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident.]

#context {
  let cap = measure(text("H")).height
  let side = 3 * (cap + par.leading.to-absolute()) + cap
  let tile = measure(_initial-tile("Q", lines: 4))
  assert(calc.abs(tile.height - side) < 0.01pt, message: repr((tile.height, side)))
  assert(calc.abs(tile.width - side) < 0.01pt)
}
