#import "/src/lib.typ": florilegium, initial
#set page(width: 12cm, height: 7cm, margin: 1cm)
#set text(size: 11pt)
#set par(justify: true)

#initial("L")[orem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.]

#initial("D", ink: red, accent: orange, seed: 3, lines: 3)[uis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident.]

#context {
  assert.eq(query(<florilegium-initial-tile>).len(), 2)
}
