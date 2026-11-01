// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! Runs the built plugin (`make wasm`) in wasmi and compares its `page` with the native one.
mod common;
use common::*;
use florilegium_core::plugin::page_cbor;
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

#[test]
fn wasm_parity() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../src/florilegium.wasm");
    let Ok(wasm) = std::fs::read(path) else {
        println!("src/florilegium.wasm is missing (run `make wasm`): wasm parity skipped");
        return;
    };
    if let Ok(dir) = std::env::var("FLO_DUMP") { std::fs::write(format!("{dir}/args.cbor"), enc(&page_args(1))).unwrap() }
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

    let e = call(&wasm, "page", &enc(&with(page_args(1), "theme", t("nope")))).unwrap_err();
    assert!(e.contains("no theme 'nope'"), "{e}");
}
