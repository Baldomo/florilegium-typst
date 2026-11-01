// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! Runs the built plugin (`make wasm`) in wasmi and compares its `page` with the native one.
mod common;
use ciborium::Value;
use common::*;
use florilegium_core::plugin::{draw_cbor, initial_cbor, ornament_cbor, page_cbor};
use wasmi::{Caller, Engine, Linker, Memory, Module, Store};

#[derive(Default)]
struct Host { args: Vec<u8>, result: Vec<u8> }

/// calls `name(args)` the way Typst does: Ok(result) when the function returns 0, Err(message) on 1
fn call(bytes: &[u8], name: &str, args: &[u8]) -> Result<Vec<u8>, String> {
    let engine = Engine::default();
    let module = Module::new(&engine, bytes).unwrap();
    let mut store = Store::new(&engine, Host { args: args.to_vec(), result: vec![] });
    let mut linker: Linker<Host> = Linker::new(&engine);
    linker.func_wrap("typst_env", "wasm_minimal_protocol_write_args_to_buffer", |mut c: Caller<'_, Host>, ptr: i32| {
        let mem: Memory = c.get_export("memory").unwrap().into_memory().unwrap();
        let a = std::mem::take(&mut c.data_mut().args);
        mem.write(&mut c, ptr as usize, &a).unwrap();
    }).unwrap();
    linker.func_wrap("typst_env", "wasm_minimal_protocol_send_result_to_host", |mut c: Caller<'_, Host>, ptr: i32, len: i32| {
        let mem: Memory = c.get_export("memory").unwrap().into_memory().unwrap();
        let mut b = vec![0u8; len as usize];
        mem.read(&c, ptr as usize, &mut b).unwrap();
        c.data_mut().result = b;
    }).unwrap();
    let inst = linker.instantiate_and_start(&mut store, &module).unwrap();
    let f = inst.get_typed_func::<i32, i32>(&store, name).unwrap();
    let code = f.call(&mut store, args.len() as i32).unwrap();
    let r = std::mem::take(&mut store.data_mut().result);
    if code == 0 { Ok(r) } else { Err(String::from_utf8_lossy(&r).into_owned()) }
}

fn svg_of(b: &[u8]) -> Vec<u8> {
    match get(&dec(b), "svg") { ciborium::Value::Bytes(s) => s.clone(), _ => panic!("svg is not bytes") }
}

fn load() -> Option<Vec<u8>> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../src/florilegium.wasm");
    let w = std::fs::read(path).ok();
    if w.is_none() { println!("src/florilegium.wasm is missing (run `make wasm`): wasm parity skipped") }
    w
}

#[test]
fn wasm_parity() {
    let Some(wasm) = load() else { return };
    if let Ok(dir) = std::env::var("FLO_DUMP") {
        std::fs::write(format!("{dir}/args.cbor"), enc(&page_args(1))).unwrap();
        std::fs::write(format!("{dir}/args7.cbor"), enc(&page_args(7))).unwrap();
    }
    let mut total = 0;
    for seed in 1..=8 {
        let args = enc(&page_args(seed));
        let native = svg_of(&page_cbor(&args).unwrap());
        let w = svg_of(&call(&wasm, "page", &args).unwrap());
        let diff = native.iter().zip(&w).filter(|(a, b)| a != b).count() + native.len().abs_diff(w.len());
        println!("seed {seed}: native {} bytes, wasm {} bytes, {diff} differ", native.len(), w.len());
        total += diff;
    }
    println!("total differing bytes: {total}");
    assert_eq!(total, 0, "the wasm page differs from the native one");
}

/// the whole result (captions, mirror, stats, canvases) of each function equals the native one
#[test]
fn wasm_matches_native_everywhere() {
    let Some(wasm) = load() else { return };
    let font = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../fonts/lmroman10-bold.otf")).unwrap();
    let cases: Vec<(&str, Value, fn(&[u8]) -> Result<Vec<u8>, String>)> = vec![
        ("page", with(page_args(2), "theme", t("math")), page_cbor),
        ("page", with(with(page_args(3), "twoside", Value::Bool(true)), "page", i(2)), page_cbor),
        ("ornament", map(vec![("name", t("flower")), ("size", i(655360)), ("angle", f(10.0)), ("count", i(2))]), ornament_cbor),
        ("draw", map(vec![("name", t("leaf")), ("size", f(1.5)), ("seed", i(4)), ("opts", map(vec![("depth", f(0.5))]))]), draw_cbor),
        ("draw", map(vec![("name", t("flower")), ("size", f(1.0)), ("seed", i(4)), ("opts", map(vec![("lift", f(0.5))]))]), draw_cbor),
        ("initial", map(vec![("letter", t("A")), ("side", i(1310720)), ("font", Value::Bytes(font.clone())), ("seed", i(5))]), initial_cbor),
    ];
    for (name, args, native) in cases {
        let a = enc(&args);
        let n = native(&a).unwrap();
        let w = call(&wasm, name, &a).unwrap();
        assert_eq!(dec(&n), dec(&w), "{name}: wasm and native results differ");
    }
    // the captions were really compared: the math page has some
    let caps = get(&dec(&page_cbor(&enc(&with(page_args(2), "theme", t("math")))).unwrap()), "captions").clone();
    assert!(matches!(&caps, Value::Array(c) if !c.is_empty()), "no captions to compare");

    // the error paths through wasm
    let e = call(&wasm, "page", &enc(&with(page_args(1), "theme", t("nope")))).unwrap_err();
    assert!(e.contains("no theme 'nope'"), "{e}");
    let e = call(&wasm, "ornament", &enc(&map(vec![("name", t("nope")), ("size", i(655360))]))).unwrap_err();
    assert!(e.contains("no primitive 'nope'"), "{e}");
    let e = call(&wasm, "initial", &enc(&map(vec![("letter", t("\u{4e2d}")), ("side", i(655360)),
        ("font", Value::Bytes(font)), ("seed", i(1))]))).unwrap_err();
    assert!(e.starts_with("florilegium: "), "{e}");
    let e = call(&wasm, "draw", &[0xff]).unwrap_err();
    assert!(e.contains("decode"), "{e}");
}
