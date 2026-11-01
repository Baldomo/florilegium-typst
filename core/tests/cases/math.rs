// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
use florilegium_core::figures;
use florilegium_core::flora::Opts;
use florilegium_core::fmt;
use florilegium_core::grow::{Caption, G};
use florilegium_core::ink::{self, Rng};
use florilegium_core::math;

fn f6(s: &mut String, x: f64) { fmt::fixed(s, x, 6) }

/// one line per caption, `%.6f` numbers, nil for what the Lua leaves unset
pub fn caption_lines(out: &mut Vec<String>, caps: &[Caption]) {
    let opt = |s: &mut String, v: Option<f64>| match v { Some(x) => f6(s, x), None => s.push_str("nil") };
    for c in caps {
        let mut s = String::new();
        f6(&mut s, c.x);
        s.push(' ');
        f6(&mut s, c.y);
        s.push(' ');
        s.push_str(c.anchor.as_deref().unwrap_or("nil"));
        s.push(' ');
        s.push_str(if c.small { "true" } else { "nil" });
        s.push(' ');
        opt(&mut s, c.scale);
        s.push(' ');
        opt(&mut s, c.rotate);
        s.push(' ');
        match c.color {
            Some(k) => { f6(&mut s, k[0]); s.push(','); f6(&mut s, k[1]); s.push(','); f6(&mut s, k[2]) }
            None => s.push_str("nil"),
        }
        s.push(' ');
        opt(&mut s, c.opacity);
        s.push(' ');
        s.push_str(&c.text);
        out.push(s);
    }
}

fn finish(mut out: String) -> String {
    out.push_str("=====\n");
    out.push_str(&ink::take_output().join("\n=====\n"));
    out
}

fn num(v: &str) -> Result<f64, String> {
    match v { "true" => Ok(1.0), "false" => Ok(0.0), _ => v.parse().map_err(|_| format!("bad number {v:?}")) }
}
fn list(v: &str) -> Result<Vec<f64>, String> { if v.is_empty() { return Ok(vec![]) } v.split(',').map(num).collect() }

pub fn run(id: &str) -> Result<String, String> {
    let bad = || format!("bad math id {id:?}");
    let (name, rest) = id.strip_prefix("math/").and_then(|r| r.split_once('/')).ok_or_else(bad)?;
    if name == "captions" {
        let p: Vec<&str> = rest.split('/').collect();
        if p.len() != 3 { return Err(bad()) }
        let seed: f64 = p[0].strip_prefix("seed=").ok_or_else(bad)?.parse().map_err(|_| bad())?;
        let spill: i32 = p[1].strip_prefix("spill=").ok_or_else(bad)?.parse().map_err(|_| bad())?;
        let case = p[2].strip_prefix("case=").ok_or_else(bad)?;
        let mut g = super::grow::geometry(seed, spill, if case == "density-0.15" { "summer" } else { case })?;
        if case == "density-0.15" { g.density = Some(0.15) }
        let r = math::page(&g)?;
        let mut out = vec![];
        caption_lines(&mut out, &r.captions);
        let mut s = out.join("\n");
        s.push_str("\n=====\n");
        s.push_str(&ink::take_output().join("\n=====\n"));
        return Ok(s);
    }
    let _: Option<&G> = None;
    let mut out = format!("{id}\n");
    ink::set_transform([ink::CM, 0.0, 0.0, ink::CM, 0.0, 0.0]);
    let (mut seed, mut big_r) = (None, None);
    let mut o = Opts::default();
    let mut any = false;
    for kv in rest.split('/') {
        let (k, v) = kv.split_once('=').ok_or_else(bad)?;
        match k {
            "seed" => seed = Some(num(v)?),
            "R" => big_r = Some(num(v)?),
            "p" => { o.p = Some(list(v)?); any = true }
            "vs" => { o.v = Some(list(v.split_once(',').map(|(a, b)| format!("{a},{b}")).ok_or_else(bad)?.as_str())?) }
            "vi" => {
                let f = figures::family(name).ok_or_else(bad)?;
                o.v = Some(f.values.get(num(v)? as usize - 1).ok_or_else(bad)?.clone());
            }
            "v" => { o.v = Some(list(v)?); any = true }
            "color" => { let c = list(v)?; o.color = Some([c[0], c[1], c[2]]); any = true }
            _ => return Err(bad()),
        }
    }
    let _ = any;
    let (r, big_r) = (Rng::new(seed.ok_or_else(bad)?), big_r.ok_or_else(bad)?);
    if figures::figure(name).is_none() && figures::family(name).is_none() && name != "hedera" { return Err(bad()) }
    if name == "hedera" {
        let (x, y) = figures::compass(&r, big_r, o.color);
        out.push_str(&format!("{} {}\n", gp(x), gp(y)));
    } else {
        math::draw(name, &r, big_r, &o)?;
    }
    out.push_str(&gp(r.u()));
    out.push('\n');
    ink::flush();
    Ok(finish(out))
}

fn gp(x: f64) -> String { let mut s = String::new(); fmt::gp(&mut s, x, 17); s }
