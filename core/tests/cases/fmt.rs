// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
use florilegium_core::fmt;

// mulberry32 (Tommy Ettinger, 2017, public domain), as ink.rng
struct Rng(u32);
impl Rng {
    fn u(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6D2B79F5);
        let mut t = self.0;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t = t ^ t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        ((t ^ (t >> 14)) as f64) / 4294967296.0
    }
}

pub fn run(id: &str) -> Result<String, String> {
    let parts: Vec<&str> = id.split('/').collect();
    if parts.len() != 3 { return Err(format!("bad fmt id {id:?}")); }
    let spec = parts[1];
    let seed: u32 = parts[2].strip_prefix("seed=").and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("bad seed in {id:?}"))?;
    let one: Box<dyn Fn(&mut String, f64)> = match spec {
        "f1" => Box::new(|o, x| fmt::fixed(o, x, 1)),
        "f2" => Box::new(|o, x| fmt::fixed(o, x, 2)),
        "f3" => Box::new(|o, x| fmt::fixed(o, x, 3)),
        "g" => Box::new(|o, x| fmt::g(o, x)),
        "g14" => Box::new(|o, x| fmt::gp(o, x, 14)),
        _ => return Err(format!("unknown fmt spec in {id:?}")),
    };
    let mut rng = Rng(seed);
    let mut out = String::new();
    for _ in 0..20000 {
        let u = rng.u();
        let e = (rng.u() * 12.0).floor() as i32 - 4;
        one(&mut out, (u - 0.5) * 10f64.powf(e as f64));
        out.push('\n');
    }
    for n in -400..=400 {
        one(&mut out, n as f64 / 20.0);
        out.push('\n');
    }
    for n in -400..=400 {
        one(&mut out, n as f64 / 200.0);
        out.push('\n');
    }
    Ok(out)
}
