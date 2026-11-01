// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
// The wasm plugin and its CBOR wrappers. Each takes a dictionary (the contract is in
// `core/src/plugin.rs`) and returns the decoded result; a failure is a compile error
// whose message starts with "florilegium: ".

#let _plugin = plugin("florilegium.wasm")

#let _call(f, args) = cbor(f(cbor.encode(args)))

/// The page's garden: `(svg, captions, mirror, stats, geometry)`.
#let page(args) = _call(_plugin.page, args)

/// An ornament: `(svg, width, height, dx, dy)`.
#let ornament(args) = _call(_plugin.ornament, args)

/// A drawing primitive: `(svg, width, height, dx, dy)`.
#let draw(args) = _call(_plugin.draw, args)

/// A drop-cap initial from a font file: `(svg, width, height, dx, dy)`.
#let initial(args) = _call(_plugin.initial, args)
