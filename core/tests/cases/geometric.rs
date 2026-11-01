// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
use florilegium_core::flora::{self, Opts, Prim};
use florilegium_core::geometric;
use florilegium_core::fmt;
use florilegium_core::ink::{self, Color, Rng};

fn say(out: &mut String, xs: &[f64]) {
    for (i, x) in xs.iter().enumerate() {
        if i > 0 { out.push(' ') }
        fmt::gp(out, *x, 17);
    }
    out.push('\n');
}
fn finish(mut out: String) -> String {
    out.push_str("=====\n");
    out.push_str(&ink::take_output().join("\n=====\n"));
    out
}

fn num(v: &str) -> Result<f64, String> { v.parse().map_err(|_| format!("bad number {v:?}")) }
fn boolean(v: &str) -> Result<bool, String> {
    match v { "true" => Ok(true), "false" => Ok(false), _ => Err(format!("bad boolean {v:?}")) }
}
fn color(v: &str) -> Result<Color, String> {
    let p: Vec<&str> = v.split(',').collect();
    if p.len() != 3 { return Err(format!("bad color {v:?}")) }
    Ok([num(p[0])?, num(p[1])?, num(p[2])?])
}

fn kvs(rest: &str) -> Vec<(&str, &str)> {
    rest.split('/').filter_map(|s| s.split_once('=')).collect()
}
fn get<'a>(kv: &[(&'a str, &'a str)], key: &str) -> Option<&'a str> {
    kv.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

fn is_scaled(name: &str) -> bool { matches!(name, "leaf" | "leaflet" | "petal") }

pub fn run(id: &str) -> Result<String, String> {
    let bad = || format!("bad geometric id {id:?}");
    let (name, rest) = id.strip_prefix("geometric/").and_then(|r| r.split_once('/')).ok_or_else(bad)?;
    let kv = kvs(rest);
    let mut out = format!("{id}\n");
    ink::set_transform([ink::CM, 0.0, 0.0, ink::CM, 0.0, 0.0]);
    let seed = || -> Result<Rng, String> { Ok(Rng::new(num(get(&kv, "seed").ok_or_else(bad)?)?)) };
    let prims = geometric::flora();
    let find = |n: &str| prims.iter().find(|(k, _)| *k == n).map(|(_, p)| p.clone());
    match name {
        "stem" => {
            let pts: Vec<[f64; 2]> = match get(&kv, "path") {
                Some("bytes") => (0..=80).map(|i| [(i as f64 / 9.0).sin(), i as f64 / 4.0]).collect(),
                Some("int") => [[0, 0], [1, 0], [2, 1], [3, 1], [3, 2], [2, 3], [2, 4], [1, 5], [0, 5], [0, 6]]
                    .iter().map(|p: &[i32; 2]| [p[0] as f64, p[1] as f64]).collect(),
                _ => return Err(bad()),
            };
            if let Some(s) = get(&kv, "scale") { ink::scale(num(s)?, None) }
            let c = get(&kv, "c").map(color).transpose()?;
            let op = get(&kv, "op").map(num).transpose()?;
            let tip = get(&kv, "tip").map(num).transpose()?;
            geometric::stem(&pts, num(get(&kv, "w0").ok_or_else(bad)?)?, num(get(&kv, "w1").ok_or_else(bad)?)?, c, op, tip);
        }
        "ghost" => {
            let lift = match get(&kv, "lift").ok_or_else(bad)? { "default" => None, v => Some(num(v)?) };
            let n = rest.strip_prefix("lift=").and_then(|r| r.split('/').nth(1)).ok_or_else(bad)?;
            let s = num(get(&kv, "s").ok_or_else(bad)?)?;
            let r = seed()?;
            let p = find(n).ok_or_else(bad)?;
            flora::ghost(lift, || match p {
                Prim::Scaled(f) => { ink::scale(s, None); f(&r, &Opts::default()) }
                Prim::Sized(f) => f(&r, s, &Opts::default()),
            });
            say(&mut out, &[ink::get_lift()]);
        }
        _ => {
            let p = find(name).ok_or_else(bad)?;
            let sizekey = if is_scaled(name) { "scale" } else { "s" };
            let mut o = Opts::default();
            for (k, v) in &kv {
                match *k {
                    "seed" => {}
                    k if k == sizekey => {}
                    "tone" => o.tone = Some(num(v)?),
                    "w" => o.w = Some(num(v)?),
                    "eye" => o.eye = Some(color(v)?),
                    "color" => o.color = Some(color(v)?),
                    "bend" => o.bend = Some(num(v)?),
                    "side" => o.side = Some(num(v)?),
                    "open" => o.open = Some(num(v)?),
                    "fork" => o.fork = Some(boolean(v)?),
                    "facing" => o.facing = Some(num(v)?),
                    "depth" => o.depth = Some(num(v)?),
                    "tilt" => o.tilt = Some(num(v)?),
                    "petals" => o.petals = Some(num(v)?),
                    "lobes" => o.lobes = Some(num(v)?),
                    "drift" => o.drift = Some(num(v)?),
                    "style" => o.style = Some(num(v)?),
                    "k" => o.k = Some(num(v)?),
                    _ => return Err(bad()),
                }
            }
            let s = num(get(&kv, sizekey).ok_or_else(bad)?)?;
            let r = seed()?;
            match p {
                Prim::Scaled(f) => { ink::scale(s, None); f(&r, &o) }
                Prim::Sized(_) if name == "hedera" => {
                    let (x, y) = geometric::hedera(&r, s, &o);
                    say(&mut out, &[x, y]);
                }
                Prim::Sized(f) => f(&r, s, &o),
            }
        }
    }
    ink::flush();
    Ok(finish(out))
}
