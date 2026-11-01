// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
use florilegium_core::flora::{self, Opts, Prim};
use florilegium_core::fmt;
use florilegium_core::ink::{self, Color, Rng};

fn say(out: &mut String, xs: &[f64]) {
    for (i, x) in xs.iter().enumerate() {
        if i > 0 { out.push(' ') }
        fmt::gp(out, *x, 17);
    }
    out.push('\n');
}
fn flat(out: &mut String, label: &str, pts: &[[f64; 2]]) {
    out.push_str(label);
    for p in pts {
        out.push(' ');
        fmt::gp(out, p[0], 17);
        out.push(' ');
        fmt::gp(out, p[1], 17);
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

pub fn run(id: &str) -> Result<String, String> {
    let bad = || format!("bad flora id {id:?}");
    let (name, rest) = id.strip_prefix("flora/").and_then(|r| r.split_once('/')).ok_or_else(bad)?;
    let kv = kvs(rest);
    let mut out = format!("{id}\n");
    ink::set_transform([ink::CM, 0.0, 0.0, ink::CM, 0.0, 0.0]);
    let seed = || -> Result<Rng, String> { Ok(Rng::new(num(get(&kv, "seed").ok_or_else(bad)?)?)) };
    let case = get(&kv, "case");
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
            flora::stem(&pts, num(get(&kv, "w0").ok_or_else(bad)?)?, num(get(&kv, "w1").ok_or_else(bad)?)?, c, op, tip);
        }
        "mottle" => {
            let r = seed()?;
            ink::scale(1.5, None);
            flora::mottle(&r, -1.0, 0.0, 1.0, 2.0, 12, 0.2, 0.2, 0.6, Some(num(get(&kv, "specks").ok_or_else(bad)?)? as i32));
            say(&mut out, &[r.u()]);
        }
        "branch" => {
            let r = seed()?;
            let pts = match case {
                Some("ahead") => flora::branch_path(&r, 0.0, 0.0, 0.0, 1.0, 1.0, 3.0, 0.0, 1.0),
                _ => flora::branch_path(&r, 1.0, 2.0, 0.0, 1.0, 0.5, -1.0, 1.0, 0.0),
            };
            flat(&mut out, "branch", &pts);
            ink::smooth(&pts, false, None);
            ink::stroke(ink::tone(0.1), 0.5, None, false, None);
            say(&mut out, &[r.u()]);
        }
        "streak" => {
            let f = |v: f64| ((v * 3.0).sin(), v * 4.0);
            flora::streak(&f, 12, 0.3, 0.02, ink::WHITE, if case == Some("a") { Some(0.8) } else { None }, case == Some("b"));
        }
        "streak3" => {
            flora::streak3(0.0, 0.0, 0.5, 1.0, 1.0, 2.0, 0.5, 3.0, 0.2, 0.01, ink::tone(0.3), if case == Some("a") { Some(0.7) } else { None }, case == Some("b"));
        }
        "walk" => {
            let k = if case == Some("a") { 30.0 } else { -20.0 };
            flat(&mut out, "walk", &flora::walk(1.0, 2.0, &|s| 90.0 + k * s, 3.0, 12));
        }
        "ghost" => {
            let lift = match get(&kv, "lift").ok_or_else(bad)? { "default" => None, v => Some(num(v)?) };
            let n = rest.strip_prefix("lift=").and_then(|r| r.split('/').nth(1)).ok_or_else(bad)?;
            let s = num(get(&kv, "s").ok_or_else(bad)?)?;
            let r = seed()?;
            let p = flora::prim(n).ok_or_else(bad)?;
            flora::ghost(lift, || match p {
                Prim::Scaled(f) => { ink::scale(s, None); f(&r, &Opts::default()) }
                Prim::Sized(f) => f(&r, s, &Opts::default()),
            });
            say(&mut out, &[ink::get_lift()]);
        }
        _ => {
            let p = flora::prim(name).ok_or_else(bad)?;
            let sizekey = if matches!(p, Prim::Scaled(_)) { "scale" } else { "s" };
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
                    "trail" => o.trail = Some(boolean(v)?),
                    "side" => o.side = Some(num(v)?),
                    "open" => o.open = Some(num(v)?),
                    "fork" => o.fork = Some(boolean(v)?),
                    "facing" => o.facing = Some(num(v)?),
                    "depth" => o.depth = Some(num(v)?),
                    "tilt" => o.tilt = Some(num(v)?),
                    "streaks" => o.streaks = Some(num(v)?),
                    "star" => o.star = Some(boolean(v)?),
                    "shape" => o.shape = Some(v.to_string()),
                    "rays" => o.rays = Some(boolean(v)?),
                    "petals" => o.petals = Some(num(v)?),
                    "lobes" => o.lobes = Some(num(v)?),
                    "heart" => o.heart = Some(boolean(v)?),
                    "drift" => o.drift = Some(num(v)?),
                    _ => return Err(bad()),
                }
            }
            let s = num(get(&kv, sizekey).ok_or_else(bad)?)?;
            let r = seed()?;
            match p {
                Prim::Scaled(f) => { ink::scale(s, None); f(&r, &o) }
                Prim::Sized(_) if name == "hedera" => {
                    let (x, y) = flora::hedera(&r, s, &o);
                    say(&mut out, &[x, y]);
                }
                Prim::Sized(f) => f(&r, s, &o),
            }
        }
    }
    ink::flush();
    Ok(finish(out))
}
