#import "/src/lib.typ": florilegium, bloom, theme, age, frame, textend
#import "/src/marks.typ"
#import "/src/garden.typ"
#set page(width: 21cm, height: 29.7cm)

#show: florilegium.with(mode: "grown", seed: 3, aging: true)
One
#context assert.eq(marks.effective(1, garden.cfg(aging: true)).age, 0.5)
