#import "/src/lib.typ": florilegium

#show: florilegium.with(mode: "grown", seed: 5, theme: "math")
#lorem(70)

#import "/src/garden.typ"
#context assert(garden.page-out(garden.cfg(mode: "grown", seed: 5, theme: "math")).captions.len() > 0)
