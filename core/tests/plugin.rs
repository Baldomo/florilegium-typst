// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
mod common;
use ciborium::Value;
use common::*;
use florilegium_core::plugin::*;

#[track_caller]
fn err(r: Result<Vec<u8>, String>) -> String { match r { Err(e) => e, Ok(_) => panic!("expected an Err") } }

#[test]
fn plugin_errors() {
    let page = |k: &str, v| enc(&with(page_args(1), k, v));
    let e = err(page_cbor(&page("theme", t("nope"))));
    assert!(e.starts_with("florilegium: ") && e.contains("no theme 'nope'"), "{e}");
    let e = err(page_cbor(&page("density", f(f64::NAN))));
    assert!(e.starts_with("florilegium: "), "{e}");
    let e = err(page_cbor(&page("page", i(0))));
    assert!(e.starts_with("florilegium: "), "{e}");
    let e = err(page_cbor(&page("marks", Value::Array(vec![map(vec![("kind", t("bloom")), ("arg", t("nope")), ("y", f(10.0))])]))));
    assert!(e.starts_with("florilegium: ") && e.contains("nope"), "{e}");
    let mut d = match get(&page_args(1), "dimens") { Value::Map(m) => m.clone(), _ => unreachable!() };
    for (k, v) in d.iter_mut() { if k.as_text() == Some("textwidth") { *v = i(0) } }
    let e = err(page_cbor(&page("dimens", Value::Map(d))));
    assert!(e.starts_with("florilegium: ") && e.contains("room"), "{e}");
    let e = err(page_cbor(&page("frame", map(vec![("k", i(5)), ("n", i(3)), ("explain", Value::Bool(false))]))));
    assert!(e.starts_with("florilegium: "), "{e}");

    let orn = |name: &str| enc(&map(vec![("name", t(name)), ("size", i(655360)), ("angle", f(0.0)), ("count", i(1))]));
    let e = err(ornament_cbor(&orn("nope")));
    assert!(e.contains("no primitive 'nope'"), "{e}");
    let e = err(ornament_cbor(&enc(&with(dec(&orn("flower")), "theme", t("nope")))));
    assert!(e.contains("no theme 'nope'"), "{e}");
    let e = err(ornament_cbor(&enc(&with(dec(&orn("flower")), "size", i(0)))));
    assert!(e.starts_with("florilegium: "), "{e}");
    let e = err(ornament_cbor(&enc(&with(dec(&orn("flower")), "angle", f(f64::INFINITY)))));
    assert!(e.starts_with("florilegium: "), "{e}");
    assert!(ornament_cbor(&orn("flower")).is_ok());

    let dr = |name: &str| enc(&map(vec![("name", t(name)), ("size", f(1.0)), ("opts", map(vec![])), ("seed", i(3))]));
    let e = err(draw_cbor(&dr("nope")));
    assert!(e.contains("no primitive 'nope'"), "{e}");
    let e = err(draw_cbor(&enc(&with(dec(&dr("flower")), "opts", map(vec![("petals", f(f64::NAN))])))));
    assert!(e.starts_with("florilegium: "), "{e}");
    let e = err(draw_cbor(&enc(&with(dec(&dr("flower")), "opts", map(vec![("p", Value::Array(vec![f(1e9)]))])))));
    assert!(e.starts_with("florilegium: "), "{e}");
    assert!(draw_cbor(&dr("flower")).is_ok());

    let font = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../fonts/lmroman10-bold.otf")).unwrap();
    let ini = |l: &str| enc(&map(vec![("letter", t(l)), ("side", i(655360)), ("font", Value::Bytes(font.clone())), ("seed", i(1))]));
    let e = err(initial_cbor(&ini("\u{4e2d}")));
    assert!(e.starts_with("florilegium: "), "{e}");
    assert!(initial_cbor(&ini("A")).is_ok());
}
