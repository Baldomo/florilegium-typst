// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
use florilegium_core::figures::{self, Family, V3};
use florilegium_core::flora::Opts;
use florilegium_core::fmt;
use florilegium_core::ink::{self, Rng};

fn say(out: &mut String, xs: &[f64]) {
    for (i, x) in xs.iter().enumerate() {
        if i > 0 { out.push(' ') }
        fmt::gp(out, *x, 17);
    }
    out.push('\n');
}
fn label_nums(out: &mut String, label: &str, xs: &[f64]) {
    out.push_str(label);
    for x in xs { out.push(' '); fmt::gp(out, *x, 17) }
    out.push('\n');
}
fn pts3(out: &mut String, label: &str, pts: &[V3]) {
    out.push_str(label);
    for q in pts { for c in q { out.push(' '); fmt::gp(out, *c, 17) } }
    out.push('\n');
}
fn finish(mut out: String) -> String {
    out.push_str("=====\n");
    out.push_str(&ink::take_output().join("\n=====\n"));
    out
}
fn num(v: &str) -> Result<f64, String> { v.parse().map_err(|_| format!("bad number {v:?}")) }
fn get<'a>(kv: &[(&'a str, &'a str)], key: &str) -> Option<&'a str> {
    kv.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

fn all_texts() -> Vec<(String, String)> {
    let mut texts = vec![];
    let r = Rng::new(1.0);
    for name in figures::FIGURE_ORDER {
        let f = figures::figure(name).unwrap();
        for _ in 0..50 { let p = (f.pick)(&r); texts.push((name.to_string(), (f.caption)(&p))) }
    }
    for name in figures::FAMILY_ORDER {
        let f = figures::family(name).unwrap();
        for _ in 0..5 {
            let p = (f.pick)(&r);
            texts.push((name.to_string(), (f.title)(&p)));
            for v in &f.values { texts.push((name.to_string(), (f.label)(v))) }
        }
    }
    texts
}

pub fn run(id: &str) -> Result<String, String> {
    let bad = || format!("bad figures id {id:?}");
    let rest = id.strip_prefix("figures/").ok_or_else(bad)?;
    let kv: Vec<(&str, &str)> = rest.split('/').filter_map(|s| s.split_once('=')).collect();
    let mut out = format!("{id}\n");
    ink::set_transform([ink::CM, 0.0, 0.0, ink::CM, 0.0, 0.0]);
    if let Some(l) = get(&kv, "lift") { ink::set_lift(num(l)?); }
    let name = rest.split('/').next().ok_or_else(bad)?;
    match rest {
        "captions" => {
            let mut s = String::new();
            for (n, t) in all_texts() { s.push_str(&format!("{n}\t{t}\n")) }
            return Ok(s);
        }
        "caption_width" => {
            let mut s = String::new();
            for (_, t) in all_texts() { fmt::gp(&mut s, figures::caption_width(&t), 17); s.push('\n') }
            return Ok(s);
        }
        "hulls" => {
            let mut s = String::new();
            for (i, so) in figures::solids().iter().enumerate() {
                pts3(&mut s, &format!("solid{}", i + 1), &so.v);
                let h = figures::hull(i);
                say(&mut s, &[h.faces.len() as f64, h.edges.len() as f64, h.radius]);
                for f in &h.faces {
                    label_nums(&mut s, "n", &f.n.map(|c| c + 0.0)); // signed zeros of the normals are invisible: normalized
                    let v: Vec<f64> = f.v.iter().map(|m| *m as f64 + 1.0).collect();
                    label_nums(&mut s, "f", &v);
                }
                for e in &h.edges {
                    label_nums(&mut s, "e", &[e.a as f64 + 1.0, e.b as f64 + 1.0, e.f.0 as f64 + 1.0, e.f.1.map_or(-1.0, |g| g as f64 + 1.0)]);
                }
            }
            let sols = figures::solids();
            let (cube, octa) = (&sols[1].v, figures::cyclic(&[[2.0, 0.0, 0.0]]));
            let cases: Vec<(&Vec<V3>, f64)> = vec![(cube, 1.0 / (2.0 + 2f64.sqrt())), (cube, 0.5), (&octa, 1.0 / 3.0), (&sols[3].v, 0.2), (&sols[4].v, 0.5)];
            for (v, t) in cases {
                let pts = figures::truncate(v, t);
                pts3(&mut s, &format!("trunc{}", pts.len()), &pts);
                let h = figures::hull_of(&pts);
                say(&mut s, &[h.faces.len() as f64, h.edges.len() as f64, h.radius]);
            }
            return Ok(s);
        }
        _ => {}
    }
    if name == "helper" {
        let what = rest.split('/').nth(1).ok_or_else(bad)?;
        let r = Rng::new(match get(&kv, "seed") { Some(s) => num(s)?, None => 1.0 });
        let req = |k: &str| -> Result<f64, String> { num(get(&kv, k).ok_or_else(bad)?) };
        match what {
            "phyllotaxis" => {
                let mut o = Opts::default();
                if let Some(s) = get(&kv, "side") { o.side = Some(num(s)?) }
                if let Some(s) = get(&kv, "drift") { o.drift = Some(num(s)?) }
                figures::phyllotaxis(&r, req("R")?, &o);
            }
            "maurer" => figures::maurer(&r, req("R")?, None),
            "fay" => figures::fay(&r, req("s")?, get(&kv, "open").map(num).transpose()?, get(&kv, "tone").map(num).transpose()?),
            "compass" => {
                let c = match get(&kv, "color") {
                    Some(c) => { let p: Vec<&str> = c.split(',').collect(); Some([num(p[0])?, num(p[1])?, num(p[2])?]) }
                    None => None,
                };
                let (x, y) = figures::compass(&r, req("s")?, c);
                say(&mut out, &[x, y]);
            }
            "vogel" => {
                for q in figures::vogel(req("N")? as usize, req("R")?, req("spin")?) { say(&mut out, &q) }
            }
            _ => return Err(bad()),
        }
        say(&mut out, &[r.u()]);
        ink::flush();
        return Ok(finish(out));
    }
    let r_ = num(get(&kv, "R").ok_or_else(bad)?)?;
    let r = Rng::new(num(get(&kv, "seed").ok_or_else(bad)?)?);
    if let Some(fam) = name.strip_prefix("family-") {
        let f: Family = figures::family(fam).ok_or_else(bad)?;
        let p = (f.pick)(&r);
        label_nums(&mut out, "p", &p);
        let v = &f.values[num(get(&kv, "v").ok_or_else(bad)?)? as usize - 1];
        if v.len() == 1 { say(&mut out, v) } else if fam != "cube" && fam != "dodeca" { label_nums(&mut out, "v", v) }
        out.push_str(&(f.label)(v));
        out.push('\n');
        (f.draw)(&r, r_, v, &p);
    } else {
        let f = figures::figure(name).ok_or_else(bad)?;
        let p = (f.pick)(&r);
        label_nums(&mut out, "p", &p);
        out.push_str(&(f.caption)(&p));
        out.push('\n');
        (f.draw)(&r, r_, &p);
    }
    say(&mut out, &[r.u()]);
    ink::flush();
    Ok(finish(out))
}
