#import "/src/geometry.typ"

#context {
  let near(a, b) = calc.abs(a.mm() - b.mm()) < 1e-6
  assert.eq(geometry.to-sp(1pt), 65781)
  assert.eq(geometry.to-sp(1in), 4736286)
  assert.eq(geometry.to-sp(68mm), 12679822)
  assert.eq(geometry.to-sp(22mm), 4102295)
  assert.eq(geometry.to-sp(8mm), 1491743)
  assert.eq(geometry.to-sp(12pt * 72 / 72.27), 786432)
  assert.eq(geometry.to-sp(30pt * 72 / 72.27), 1966080)
  assert.eq(geometry.to-sp(-8mm), -1491743)
  // no short decimal in any unit: rounded
  assert.eq(geometry.to-sp(1.2345678901pt), calc.round(1.2345678901 * 72.27 / 72 * 65536))

  let m = geometry.resolve-margin(auto, 210mm, 297mm)
  assert(m.values().all(v => near(v, 25mm)), message: repr(m))
  let m = geometry.resolve-margin(auto, 297mm, 210mm)
  assert(m.values().all(v => near(v, 25mm)), message: repr(m))
  let m = geometry.resolve-margin(20mm, 210mm, 297mm)
  assert(m.values().all(v => near(v, 20mm)))
  let d = (inside: 1cm, outside: 2cm, y: 3cm)
  let m = geometry.resolve-margin(d, 210mm, 297mm)
  assert(near(m.left, 1cm) and near(m.right, 2cm) and near(m.top, 3cm) and near(m.bottom, 3cm), message: repr(m))
  let m = geometry.resolve-margin(d, 210mm, 297mm, even: true)
  assert(near(m.left, 2cm) and near(m.right, 1cm), message: repr(m))
  let m = geometry.resolve-margin(d, 210mm, 297mm, binding: right)
  assert(near(m.left, 2cm) and near(m.right, 1cm), message: repr(m))
  let m = geometry.resolve-margin(d, 210mm, 297mm, even: true, binding: right)
  assert(near(m.left, 1cm) and near(m.right, 2cm), message: repr(m))
  let m = geometry.resolve-margin((x: 1cm, top: 5%, rest: 2cm), 210mm, 297mm)
  assert(near(m.left, 1cm) and near(m.right, 1cm) and near(m.top, 14.85mm) and near(m.bottom, 2cm), message: repr(m))
  let m = geometry.resolve-margin((left: 1cm, bottom: 1cm), 210mm, 297mm)
  assert(near(m.left, 1cm) and near(m.right, 25mm), message: repr(m))
}
