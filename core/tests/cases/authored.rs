// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
use florilegium_core::authored;
use florilegium_core::grow::G;
use florilegium_core::ink;

pub fn run(id: &str) -> Result<String, String> {
    let bad = || format!("bad authored id {id:?}");
    let rest = id.strip_prefix("authored/seed=").ok_or_else(bad)?;
    let (seed, case) = rest.split_once("/case=").ok_or_else(bad)?;
    let (case, theme) = match case.split_once("/theme=") { Some((c, t)) => (c, Some(t.to_string())), None => (case, None) };
    let seed: f64 = seed.parse().map_err(|_| bad())?;
    let mut g = G { pw: 21.0, ph: 29.7, textleft: 6.8, textright: 18.8, ytop: 25.4, ybot: 3.0, ..Default::default() };
    g.colright = g.textleft - 0.4;
    g.headrule = g.ytop + 0.7;
    g.footy = 1.9;
    g.seed = seed;
    g.theme = theme;
    match case {
        "plain" => {}
        "novis" => g.visitors = Some(false),
        "mirror" => g.mirror = true,
        _ => return Err(bad()),
    }
    let r = authored::page(&g)?;
    let mut out = vec![];
    super::math::caption_lines(&mut out, &r.captions);
    let mut s = out.join("\n");
    s.push_str("\n=====\n");
    s.push_str(&ink::take_output().join("\n=====\n"));
    Ok(s)
}
