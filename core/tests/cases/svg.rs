// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! The Rust twin of the svg, captions, frame, ornament and draw families of
//! tools/oracle/cases.lua. After the family, an id is a list of `key=value` or bare `flag`
//! segments: for svg and captions they become `--key value` / `--flag` of the command line.
use florilegium_core::flora::Opts;
use florilegium_core::fmt;
use florilegium_core::glue;
use florilegium_core::svg;

fn argv(rest: &str) -> Vec<String> {
    let mut v = vec![];
    for seg in rest.split('/').filter(|s| !s.is_empty()) {
        match seg.split_once('=') {
            Some((k, val)) => { v.push(format!("--{k}")); v.push(val.to_string()) }
            None => v.push(format!("--{seg}")),
        }
    }
    v
}

fn kv<'a>(rest: &'a str, key: &str) -> Option<&'a str> {
    rest.split('/').find_map(|s| s.strip_prefix(key).and_then(|r| r.strip_prefix('=')))
}

fn frame_opts(name: &str) -> Option<&'static [&'static str]> {
    Some(match name {
        "summer" => &[],
        "spring" => &["season=0.05"],
        "autumn" => &["season=0.95"],
        "geometric" => &["theme=geometric"],
        "geometric-autumn-mirrored" => &["theme=geometric", "season=0.95", "twoside", "page=2"],
        "density-0.4" => &["density=0.4"],
        "density-0.05" => &["density=0.05"],
        "math" => &["theme=math"],
        "math-mirrored-density-0.3" => &["theme=math", "twoside", "page=2", "density=0.3"],
        _ => return None,
    })
}

fn f6(s: &mut String, x: f64) { fmt::fixed(s, x, 6) }

fn page_args(rest: &str) -> Result<glue::PageArgs, String> {
    Ok(svg::options(&argv(rest))?.page)
}

pub fn run(id: &str) -> Result<String, String> {
    let (family, rest) = id.split_once('/').ok_or("bad id")?;
    let bad = || format!("bad {family} id {id:?}");
    match family {
        "svg" => Ok(svg::page(&page_args(rest)?)?.svg),
        "captions" => {
            let r = svg::page(&page_args(rest)?)?;
            let mut out = String::new();
            for c in &r.captions {
                f6(&mut out, c.x_cm); out.push(' ');
                f6(&mut out, c.y_cm); out.push(' ');
                out.push_str(&c.anchor); out.push(' ');
                f6(&mut out, c.size_pt); out.push(' ');
                f6(&mut out, c.rotate); out.push(' ');
                out.push_str(&format!("#{:02x}{:02x}{:02x} ", c.color[0], c.color[1], c.color[2]));
                f6(&mut out, c.opacity); out.push(' ');
                out.push_str(&c.tex);
                out.push('\n');
            }
            if !r.stats.is_empty() { out.push_str(&r.stats); out.push('\n') }
            if out.is_empty() { out.push('\n') }
            Ok(out)
        }
        "frame" => {
            let p: Vec<&str> = rest.split('/').collect();
            let name = p.first().and_then(|s| s.strip_prefix("case=")).ok_or_else(bad)?;
            let k: i64 = p.get(1).and_then(|s| s.strip_prefix("k=")).and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            let n: i64 = p.get(2).and_then(|s| s.strip_prefix("n=")).and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            let explain = match p.get(3) { None => false, Some(&"explain") => true, _ => return Err(bad()) };
            let opts = frame_opts(name).ok_or_else(bad)?;
            let mut segs = vec!["seed=2"];
            segs.extend_from_slice(opts);
            let mut a = page_args(&segs.join("/"))?;
            a.frame = Some(glue::Frame { k, n, explain });
            Ok(svg::page(&a)?.svg)
        }
        "ornament" | "draw" => {
            let (name, rest) = rest.split_once('/').ok_or_else(bad)?;
            let theme = kv(rest, "theme");
            let (ink, accent) = (kv(rest, "ink").unwrap_or(""), kv(rest, "accent").unwrap_or(""));
            let num = |k: &str| kv(rest, k).map(|v| v.parse::<f64>().map_err(|_| bad())).transpose();
            if family == "ornament" {
                let size: i64 = kv(rest, "size").and_then(|s| s.parse().ok()).ok_or_else(bad)?;
                let angle = num("angle")?.unwrap_or(0.0);
                let seed = kv(rest, "seed").map(|s| s.parse::<i64>().map_err(|_| bad())).transpose()?;
                let count: i64 = kv(rest, "count").map_or(Ok(1), |s| s.parse().map_err(|_| bad()))?;
                // the k-th ornament of a document: the earlier ones leave nothing to compare
                glue::ornament(name, size, ink, accent, angle, seed, count, theme)
            } else {
                let seed: i64 = kv(rest, "seed").and_then(|s| s.parse().ok()).ok_or_else(bad)?;
                let seed = seed.wrapping_mul(7919).wrapping_add(131); // Lua's seed(1, s); draw forks it
                let o = Opts {
                    lift: num("lift")?, side: num("side")?, drift: num("drift")?,
                    trail: (kv(rest, "trail") == Some("false")).then_some(false),
                    ..Default::default()
                };
                let s = num("s")?.ok_or_else(bad)?;
                if name == "ghost" {
                    glue::draw_ghost(kv(rest, "inner").ok_or_else(bad)?, s, &o, ink, accent, theme, seed)
                } else {
                    glue::draw(name, s, &o, ink, accent, theme, seed)
                }
            }
        }
        _ => Err(bad()),
    }
}
