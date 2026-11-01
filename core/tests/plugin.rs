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
    let with_ink = |c: &str| enc(&with(with(dec(&orn("flower")), "ink", t(c)), "accent", t(c)));
    assert_eq!(ornament_cbor(&with_ink("#2456C4")).unwrap(), ornament_cbor(&with_ink("2456C4")).unwrap());

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

    // malformed input
    for f in [page_cbor, ornament_cbor, draw_cbor, initial_cbor] {
        let e = err(f(&[0xff, 0xff, 0xff]));
        assert!(e.starts_with("florilegium: ") && e.contains("decode"), "{e}");
        let e = err(f(&enc(&Value::Array(vec![i(1)]))));
        assert!(e.contains("must be a dictionary"), "{e}");
    }
    let e = err(page_cbor(&page("mode", t("sideways"))));
    assert!(e.contains("mode is"), "{e}");
    let e = err(draw_cbor(&enc(&with(dec(&dr("flower")), "opts", map(vec![("petalz", i(5))])))));
    assert!(e.contains("unknown option 'petalz'"), "{e}");

    // caps
    let opt = |k: &str, v| enc(&with(dec(&dr("sprig")), "opts", map(vec![(k, v)])));
    for (k, v) in [("depth", i(50)), ("depth", i(-1)), ("petals", i(100000)), ("lobes", i(101)), ("streaks", f(1e6))] {
        let e = err(draw_cbor(&opt(k, v)));
        assert!(e.starts_with("florilegium: ") && e.contains(k), "{e}");
    }
    assert!(draw_cbor(&opt("depth", i(8))).is_ok());
    let hil = enc(&map(vec![("name", t("hilbert")), ("size", f(2.0)), ("seed", i(1)), ("theme", t("math")),
        ("opts", map(vec![("p", Value::Array(vec![i(1), i(30)]))]))]));
    let e = err(draw_cbor(&hil));
    assert!(e.contains("p[1]"), "{e}");
    let e = err(draw_cbor(&enc(&with(dec(&dr("flower")), "size", f(51.0)))));
    assert!(e.contains("at most 50"), "{e}");
    let e = err(ornament_cbor(&enc(&with(dec(&orn("flower")), "size", i(100_000_001)))));
    assert!(e.contains("at most"), "{e}");
    let e = err(initial_cbor(&enc(&with(dec(&ini("A")), "side", i(1 << 40)))));
    assert!(e.contains("at most"), "{e}");
    let mut d = match get(&page_args(1), "dimens") { Value::Map(m) => m.clone(), _ => unreachable!() };
    for (k, v) in d.iter_mut() { if k.as_text() == Some("paperwidth") { *v = i(1 << 29) } }
    let e = err(page_cbor(&page("dimens", Value::Map(d))));
    assert!(e.contains("150 cm"), "{e}");
    let e = err(page_cbor(&page("marks", Value::Array(vec![map(vec![("kind", t("heading")), ("arg", i(1)), ("y", f(1.0))]); 1001]))));
    assert!(e.contains("too many marks"), "{e}");
}

#[test]
fn draw_with_lift_takes_the_ghost_path() {
    let base = || map(vec![("name", t("flower")), ("size", f(1.0)), ("seed", i(3)), ("opts", map(vec![]))]);
    let plain = dec(&draw_cbor(&enc(&base())).unwrap());
    let lifted = dec(&draw_cbor(&enc(&with(base(), "opts", map(vec![("lift", f(0.6))])))).unwrap());
    assert_ne!(get(&plain, "svg"), get(&lifted, "svg"));
    let w = get(&lifted, "width").as_float().unwrap();
    assert!(w > 0.0 && get(&lifted, "dx").as_float().unwrap() > 0.0);
}

#[test]
fn page_returns_its_geometry() {
    let r = dec(&page_cbor(&enc(&page_args(1))).unwrap());
    let g = get(&r, "geometry");
    for k in ["pw", "ph", "textleft", "textright", "ytop", "ybot", "colright", "headrule", "footy"] {
        assert!(get(g, k).as_float().is_some(), "geometry.{k}");
    }
    assert!(get(g, "textright").as_float().unwrap() > get(g, "textleft").as_float().unwrap());
}
