// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
#![allow(dead_code)]
use ciborium::Value;
use florilegium_core::glue::{Mode, PageArgs};

pub fn t(s: &str) -> Value { Value::Text(s.into()) }
pub fn i(n: i64) -> Value { Value::Integer(n.into()) }
pub fn f(x: f64) -> Value { Value::Float(x) }
pub fn map(kv: Vec<(&str, Value)>) -> Value { Value::Map(kv.into_iter().map(|(k, v)| (t(k), v)).collect()) }
pub fn enc(v: &Value) -> Vec<u8> {
    let mut b = vec![];
    ciborium::into_writer(v, &mut b).unwrap();
    b
}
pub fn dec(b: &[u8]) -> Value { ciborium::from_reader(b).unwrap() }
pub fn get<'a>(v: &'a Value, k: &str) -> &'a Value {
    match v {
        Value::Map(m) => m.iter().find(|(a, _)| a.as_text() == Some(k)).map(|(_, v)| v).unwrap_or_else(|| panic!("no key {k}")),
        _ => panic!("not a map"),
    }
}

pub fn dimens(a: &PageArgs) -> Value {
    let d = &a.dimens;
    map(vec![
        ("paperwidth", i(d.paperwidth)), ("paperheight", i(d.paperheight)), ("textwidth", i(d.textwidth)),
        ("textheight", i(d.textheight)), ("oddsidemargin", i(d.oddsidemargin)), ("evensidemargin", i(d.evensidemargin)),
        ("topmargin", i(d.topmargin)), ("headheight", i(d.headheight)), ("headsep", i(d.headsep)),
        ("footskip", i(d.footskip)), ("hoffset", i(d.hoffset)),
    ])
}

/// the page arguments of `svg.lua --seed N` (the default A4 layout) as the plugin takes them
pub fn page_args(seed: i64) -> Value {
    let cli = florilegium_core::svg::options(&["--seed".to_string(), seed.to_string()]).unwrap();
    let a = cli.page;
    map(vec![
        ("page", i(a.page)), ("twoside", Value::Bool(a.twoside)), ("dimens", dimens(&a)), ("seed", i(seed)),
        ("mode", t(if a.mode == Mode::Grown { "grown" } else { "authored" })), ("marks", Value::Array(vec![])),
        ("density", f(1.0)), ("intensity", f(1.0)), ("visitors", Value::Bool(true)), ("ink", t("")), ("accent", t("")),
    ])
}

pub fn with(base: Value, k: &str, v: Value) -> Value {
    match base {
        Value::Map(mut m) => {
            m.retain(|(a, _)| a.as_text() != Some(k));
            m.push((t(k), v));
            Value::Map(m)
        }
        _ => panic!(),
    }
}
