// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
use florilegium_core::fmt;
use florilegium_core::initial::{self, Glyph, Seg, Shapes};
use florilegium_core::ink::{self, Rng};
use std::collections::HashMap;

/// the dump of tools/oracle/shapes.lua
pub fn dump() -> Shapes {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/lmroman10-bold.shapes");
    let text = std::fs::read_to_string(path).expect("lmroman10-bold.shapes");
    let mut units = 1000.0;
    let mut glyphs = HashMap::new();
    let mut cur: Option<(u32, Glyph)> = None;
    for line in text.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        let n = |i: usize| -> f64 { w[i].parse().unwrap() };
        match w[0] {
            "units" => units = n(1),
            "glyph" => {
                let bb = if w[2] == "nobox" { [0.0; 4] } else { [n(2), n(3), n(4), n(5)] };
                cur = Some((w[1].parse().unwrap(), Glyph { boundingbox: bb, segments: vec![] }));
            }
            "end" => { let (u, g) = cur.take().unwrap(); glyphs.insert(u, g); }
            op => {
                let seg = match op {
                    "m" => Seg::M(n(1), n(2)),
                    "l" => Seg::L(n(1), n(2)),
                    "c" => Seg::C(n(1), n(2), n(3), n(4), n(5), n(6)),
                    "q" => Seg::Q(n(1), n(2), n(3), n(4)),
                    _ => panic!("bad dump line {line:?}"),
                };
                cur.as_mut().unwrap().1.segments.push(seg);
            }
        }
    }
    Shapes { units, glyphs, file: initial::DEFAULT_FONT.to_string() }
}

pub fn run(id: &str) -> Result<String, String> {
    let bad = || format!("bad initial id {id:?}");
    let rest = id.strip_prefix("initial/").ok_or_else(bad)?;
    let (letter, rest2) = rest.split_once("/S=").ok_or_else(bad)?;
    let (s, seed) = rest2.split_once("/seed=").ok_or_else(bad)?;
    let num = |v: &str| v.parse::<f64>().map_err(|_| bad());
    ink::set_transform([ink::CM, 0.0, 0.0, ink::CM, 0.0, 0.0]);
    let r = Rng::new(num(seed)?);
    if let Err(e) = initial::draw(&r, num(s)?, letter, &dump()) {
        return Ok(format!("{id}\nerror {e}\n"));
    }
    let mut out = format!("{id}\n");
    fmt::gp(&mut out, r.u(), 17);
    out.push('\n');
    ink::flush();
    out.push_str("=====\n");
    out.push_str(&ink::take_output().join("\n=====\n"));
    Ok(out)
}
