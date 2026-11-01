// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
use florilegium_core::fmt;
use florilegium_core::ink::{self, Color, Rng, Tf};
use std::cell::RefCell;
use std::rc::Rc;

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
fn c3(out: &mut String, c: Color) { say(out, &c) }

// the xcolor values of svg.lua's html(), as the Lua computes them
fn html(h: &str) -> Result<Color, String> {
    Ok(match h {
        "2456C4" => [0.14119000000000001, 0.33724999999999999, 0.76863000000000004],
        "B4662F" => [0.70589000000000002, 0.40000000000000002, 0.18431],
        "5B2A1F" => [0.35686000000000001, 0.16472000000000001, 0.12157999999999999],
        "2F6B3A" => [0.18431, 0.41961999999999999, 0.22745000000000001],
        "808080" => [0.50195000000000001; 3],
        "000000" => [0.0; 3],
        _ => return Err(format!("unknown color {h}")),
    })
}

fn ramp_lines(out: &mut String) {
    for i in 0..=100 { c3(out, ink::tone(i as f64 / 100.0)) }
    for i in -10..=10 { c3(out, ink::accent(i as f64 / 10.0)) }
    let mut o2 = String::new();
    ink::in_accent(|| for i in 0..=100 { c3(&mut o2, ink::tone(i as f64 / 100.0)) });
    out.push_str(&o2);
}

const PTS: [[f64; 2]; 5] = [[0.0, 0.0], [1.0, 0.5], [2.0, 0.2], [3.0, 1.0], [3.5, 2.0]];

fn tf(out: &mut String, t: Tf) { say(out, &t) }

pub fn run(id: &str) -> Result<String, String> {
    let bad = || format!("bad ink id {id:?}");
    let parts: Vec<&str> = id.splitn(3, '/').collect();
    if parts.len() != 3 { return Err(bad()); }
    let (kind, rest) = (parts[1], parts[2]);
    let mut out = String::new();
    let arg = |key: &str| rest.strip_prefix(key).ok_or_else(bad);
    match kind {
        "rng" => {
            let seed: f64 = arg("seed=")?.parse().map_err(|_| bad())?;
            let r = Rng::new(seed);
            for _ in 0..2000 { say(&mut out, &[r.u()]) }
            for _ in 0..2000 { say(&mut out, &[r.range(-3.0, 5.0)]) }
            for _ in 0..2000 { say(&mut out, &[r.int(1.0, 6.0)]) }
            for _ in 0..2000 { say(&mut out, &[r.normal()]) }
            for _ in 0..500 { say(&mut out, &[r.sym(2.5)]) }
            for _ in 0..500 { say(&mut out, &[if r.chance(0.3) { 1.0 } else { 0.0 }]) }
            for _ in 0..500 { say(&mut out, &[r.pick(&[10.0, 20.0, 30.0, 40.0, 50.0])]) }
            let s = r.save();
            let (a, b) = (r.u(), r.u());
            say(&mut out, &[s as f64, a, b]);
            r.load(s);
            say(&mut out, &[r.u()]);
            let mut r = r;
            for _ in 0..200 { r = r.fork(); say(&mut out, &[r.u()]) }
            Ok(out)
        }
        "noise" => {
            let seed: f64 = arg("seed=")?.parse().map_err(|_| bad())?;
            let r = Rng::new(seed);
            let f = ink::fbm1(&r, 3);
            for i in 0..500 { say(&mut out, &[f(i as f64 * 0.37 - 20.0)]) }
            let n = ink::noise1(&r);
            for i in 0..100 { say(&mut out, &[n(i as f64 * 0.91 - 40.0)]) }
            Ok(out)
        }
        "ramp" => {
            let r = arg("ink=")?;
            let (i, a) = r.split_once("/accent=").ok_or_else(bad)?;
            let c = html(i)?;
            ink::set_ink(c[0], c[1], c[2]);
            let c = html(a)?;
            ink::set_accent(c[0], c[1], c[2]);
            ramp_lines(&mut out);
            Ok(out)
        }
        "pale" => {
            ink::set_pale(arg("v=")?.parse().map_err(|_| bad())?);
            ramp_lines(&mut out);
            Ok(out)
        }
        "draw" => {
            draw(arg("case=")?, &mut out)?;
            Ok(finish(out))
        }
        _ => Err(bad()),
    }
}

fn draw(name: &str, out: &mut String) -> Result<(), String> {
    use ink::*;
    match name {
        "paths" => {
            layer(3);
            moveto(1.0, 1.0); lineto(3.0, 1.0); lineto(3.0, 3.0); close(); fill(tone(0.3), None, false);
            moveto(0.0, 0.0); curveto(1.0, 2.0, 2.0, 2.0, 3.0, 0.0); stroke(tone(0.1), 0.5, None, false, None);
            poly(&[[0.0, 0.0], [1.0, 0.5], [2.0, 0.0]], false); stroke(tone(0.2), 0.25, Some(0.5), false, Some("[1.2 1.6] 0"));
            poly(&[[0.0, 0.0], [1.0, 0.5], [2.0, 0.0]], true); fill(accent(0.3), Some(0.4), true);
            smooth(&[[0.0, 0.0], [1.0, 1.0]], true, None); stroke(tone(0.4), 0.3, None, false, None);
            smooth(&[[0.0, 0.0], [1.0, 1.0], [2.0, 0.0]], false, None); stroke(tone(0.4), 0.3, Some(1.0), false, None);
            smooth(&PTS, false, None); fill(tone(0.5), Some(0.85), false);
            smooth(&PTS, true, Some(0.7)); stroke(tone(0.35), 0.8, Some(0.999), true, None);
            smooth(&PTS, false, None); stroke(accent(-0.4), 1.25, Some(0.304), false, None);
            circle(2.0, 2.0, 0.75); stroke(WHITE, 1.0, Some(1.0), true, None);
            circle(2.0, 2.0, 0.75); clip();
            flood(tone(0.6), -1.0, -1.0, 5.0, 5.0);
            circle(2.5, 2.0, 0.4); fill(PAPER, Some(0.5), false);
            unclip();
            moveto(9.0, 9.0); lineto(1.0, 1.0); discard();
            layer(7); circle(1.0, 1.0, 1.0); fill(tone(0.9), Some(0.123456), false);
            layer(1); poly(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]], true); fill(tone(0.05), None, false);
            layer(5); poly(&[[0.0, 0.0], [-1.0, 0.0], [-1.0, -1.0]], true); fill(tone(0.15), Some(1.0), false);
            let (a, b) = (layer(2), layer(5));
            say(out, &[a as f64, b as f64]);
            say(out, &[flush() as f64]);
            say(out, &[flush() as f64]);
        }
        "transforms" => {
            tf(out, get_transform());
            tf(out, page_transform(None));
            tf(out, page_transform(Some(21.0)));
            push();
            translate(1.0, 2.0); rotate(30.0); scale(2.0, None);
            tf(out, get_transform());
            say(out, &[unit()]);
            let (x, y) = to_page(0.5, 0.25);
            say(out, &[x, y]);
            poly(&[[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]], true); fill(tone(0.3), None, false);
            push();
            scale(1.0, Some(3.0)); rotate(-75.5); translate(-0.3, 0.2);
            tf(out, get_transform());
            say(out, &[unit()]);
            let (x, y) = to_page(-1.0, 2.0);
            say(out, &[x, y]);
            circle(0.0, 0.0, 0.5); stroke(tone(0.2), 0.3, None, false, None);
            pop();
            tf(out, get_transform());
            pop();
            tf(out, get_transform());
            set_transform(page_transform(Some(21.0)));
            poly(&[[1.0, 1.0], [2.0, 1.0], [2.0, 2.0]], true); fill(tone(0.25), None, false);
            let (x, y) = to_page(3.0, 4.0);
            say(out, &[x, y]);
            set_transform(page_transform(None));
            poly(&[[1.0, 1.0], [2.0, 1.0], [2.0, 2.0]], true); fill(tone(0.25), None, false);
            flush();
        }
        "geometry" => {
            let deg = [[0.5, 0.5], [0.5, 0.5], [1.5, 1.5], [1.5, 1.5]];
            flat(out, "normals", &normals(&PTS));
            flat(out, "normals-degenerate", &normals(&deg));
            let (s, l) = arclen(&PTS);
            let mut v = vec![l]; v.extend(s); say(out, &v);
            let (s, l) = arclen(&[[1.0, 1.0]]);
            let mut v = vec![l]; v.extend(s); say(out, &v);
            let rb = ribbon(&PTS, &|u| 0.2 + 0.3 * u);
            flat(out, "ribbon", &rb);
            poly(&rb, true); fill(tone(0.3), None, false);
            flat(out, "ribbon-point", &ribbon(&[[1.0, 1.0]], &|u| 0.5 + u));
            taper(0.0, 0.0, 1.0, 2.0, 2.0, 2.0, 3.0, 0.0, 0.4, 0.05); fill(tone(0.2), None, false);
            taper(0.0, 0.0, 0.0, 0.0, 2.0, 2.0, 2.0, 2.0, 0.4, 0.05); fill(tone(0.2), None, false);
            flat(out, "catmull", &catmull(&PTS, 4));
            flat(out, "catmull2", &catmull(&[[0.0, 0.0], [2.0, 3.0]], 3));
            flat(out, "catmull1", &catmull(&[[5.0, 7.0]], 3));
            let b3 = bezier3(0.0, 0.0, 1.0, 2.0, 2.0, 2.0, 3.0, 0.0);
            for v in [0.0, 0.3, 1.0] { let (x, y) = b3(v); say(out, &[x, y]) }
            flat(out, "sample", &sample(&b3, 8, None, None));
            flat(out, "sample-range", &sample(&b3, 5, Some(0.2), Some(0.8)));
            let spine = |v: f64| (v * 2.0, (v * 3.0).sin() * 0.5);
            let wl = |v: f64| 0.3 * (v * std::f64::consts::PI).sin();
            let wr = |v: f64| 0.25 * (v * std::f64::consts::PI).sin() + 0.05;
            let (bl, c, nrm) = blade(&spine, 6, &wl, &wr, None);
            flat(out, "blade", &bl); flat(out, "blade-c", &c); flat(out, "blade-n", &nrm);
            smooth(&bl, true, None); fill(tone(0.4), None, false);
            let edge = |v: f64, side: f64| side * 0.1 * (v * 9.0).cos() - 0.2;
            let (bl, c, nrm) = blade(&spine, 7, &wl, &wr, Some(&edge));
            flat(out, "blade-edge", &bl); flat(out, "blade-edge-c", &c); flat(out, "blade-edge-n", &nrm);
            let r = Rng::new(5.0);
            flat(out, "blob", &blob(&r, 1.0, 2.0, 1.5, 1.0, 0.3, None));
            flat(out, "blob5", &blob(&r, -1.0, 0.5, 0.5, 1.5, 0.2, Some(5)));
            say(out, &[r.u()]);
            flush();
        }
        "recording" => {
            let t = |e: &Rc<RefCell<ink::EventData>>, from, to, overlay| Tag { event: e.clone(), from, to, overlay };
            poly(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]], true); fill(tone(0.1), None, false);
            record(true);
            let e1 = Rc::new(RefCell::new(ink::EventData { t0: 0.0, dur: 1.0, ..Default::default() }));
            let e2 = Rc::new(RefCell::new(ink::EventData { t0: 1.0, dur: 2.0, ..Default::default() }));
            say(out, &[if tag(Some(t(&e1, 0.0, None, false))).is_none() { 1.0 } else { 0.0 }]);
            circle(1.0, 1.0, 1.0); fill(tone(0.2), None, false);
            let old = tag(Some(t(&e1, 0.5, Some(1.0), false))).unwrap();
            say(out, &[old.from, if old.to.is_none() { 1.0 } else { 0.0 }]);
            circle(2.0, 1.0, 1.0); fill(tone(0.3), None, false);
            layer(2);
            tag(Some(t(&e2, 0.0, None, true)));
            circle(3.0, 1.0, 1.0); fill(tone(0.4), None, false);
            tag(None);
            circle(4.0, 1.0, 1.0); fill(tone(0.5), None, false);
            layer(5);
            let old = tag(Some(t(&e2, 1.0, None, false)));
            circle(5.0, 1.0, 1.0); fill(tone(0.6), None, false);
            tag(old);
            circle(6.0, 1.0, 1.0); fill(tone(0.7), None, false);
            tag(Some(t(&e2, 0.0, Some(0.5), false)));
            record(true);
            circle(7.0, 1.0, 1.0); fill(tone(0.8), None, false);
            tag(Some(t(&e1, 0.0, Some(0.25), false)));
            circle(8.0, 1.0, 1.0); fill(tone(0.85), None, false);
            let rec = take_recording();
            say(out, &[rec.keys.len() as f64, rec.keys[0] as f64, rec.keys[1] as f64]);
            circle(9.0, 9.0, 1.0); fill(tone(0.9), None, false);
            say(out, &[flush() as f64]);
            say(out, &[emit(&rec, &|_| true) as f64]);
            let now = 1.5;
            say(out, &[emit(&rec, &|tag| {
                if tag.overlay { return false; }
                let e = tag.event.borrow();
                let p = (now - e.t0) / e.dur;
                p >= tag.from && tag.to.map_or(true, |to| p < to)
            }) as f64]);
            say(out, &[emit(&rec, &|_| false) as f64]);
            record(true); record(false);
            circle(0.0, 0.0, 1.0); fill(tone(0.1), None, false);
            say(out, &[flush() as f64]);
        }
        "colors" => {
            say(out, &[get_lift()]);
            let old = set_lift(0.3);
            say(out, &[old, get_lift()]);
            for i in 0..=10 { c3(out, tone(i as f64 / 10.0)) }
            circle(1.0, 1.0, 1.0); fill(tone(0.2), None, false);
            say(out, &[set_pale(0.5)]);
            for i in 0..=10 { c3(out, tone(i as f64 / 10.0)) }
            c3(out, accent(0.4));
            set_accent(0.2, 0.5, 0.3);
            let mut inner = String::new();
            in_accent(|| {
                for i in 0..=10 { c3(&mut inner, tone(i as f64 / 10.0)) }
                circle(2.0, 1.0, 1.0); fill(tone(0.4), None, false);
                in_accent(|| { circle(3.0, 1.0, 1.0); fill(tone(0.5), None, false); });
                circle(4.0, 1.0, 1.0); fill(tone(0.6), None, false);
            });
            out.push_str(&inner);
            circle(5.0, 1.0, 1.0); fill(tone(0.6), None, false);
            set_accent(0.9, 0.2, 0.1);
            in_accent(|| { circle(6.0, 1.0, 1.0); fill(tone(0.6), None, false); });
            in_accent(|| { circle(7.0, 1.0, 1.0); fill(tone(0.7), None, false); });
            let c = html("5B2A1F")?;
            set_ink(c[0], c[1], c[2]);
            in_accent(|| { set_ink(0.1, 0.6, 0.2); circle(8.0, 1.0, 1.0); fill(tone(0.7), None, false); });
            circle(9.0, 1.0, 1.0); fill(tone(0.7), None, false);
            in_accent(|| { circle(10.0, 1.0, 1.0); fill(tone(0.7), None, false); });
            flush();
        }
        _ => return Err(format!("bad ink draw case {name:?}")),
    }
    Ok(())
}
