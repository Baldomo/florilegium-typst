// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! florilegium-theme-geometric.lua: the same plant, cut from flat shapes with straight edges.
//!
//! grow.rs draws every primitive through the current theme, so the garden grows exactly
//! as it does with flora; only the drawing changes. Stems are straight runs, leaves kites
//! or toothed polygons folded into a light and a dark facet, flowers rosettes of rhombi or
//! layered stars. Colors are flat tones of the ink; copper marks the centers. One motif
//! is added: crystals (Platonic solids, flat-shaded like cut stones).
use crate::figures;
use crate::flora::{Opts, Prim};
use crate::grow::{Flora, GPrim, Motif, StemFn, ThemeDef, L_BLOOM};
use crate::ink::{self, Color, Rng, PAPER, WHITE};
use std::f64::consts::PI;
use std::rc::Rc;

type Pt = [f64; 2];

const RAD: f64 = PI / 180.0;

fn phi() -> f64 { (1.0 + 5f64.sqrt()) / 2.0 }
fn lerp(a: f64, b: f64, t: f64) -> f64 { a + (b - a) * t }
fn tone(t: f64) -> Color { ink::tone(t) }
fn ghost() -> bool { ink::get_lift() > 0.3 }
fn fine(u: Option<f64>) -> bool { ink::unit() > u.unwrap_or(0.25) && !ghost() }

fn stroke(c: Color, w: f64, op: Option<f64>) { ink::stroke(c, w, op, false, None) }

/// a flat facet; white seams between facets; points turned by a (radians)
fn facet(pts: &[Pt], c: Color, op: Option<f64>) {
    ink::poly(pts, true);
    ink::fill(c, op, false);
}
fn turn(pts: &[Pt], a: f64) -> Vec<Pt> {
    let (c, s) = (a.cos(), a.sin());
    pts.iter().map(|p| [p[0] * c - p[1] * s, p[0] * s + p[1] * c]).collect()
}
fn ngon(n: f64, r: f64, a0: f64, cx: f64, cy: f64) -> Vec<Pt> {
    let mut out = vec![];
    let mut k = 0.0;
    while k <= n - 1.0 {
        let a = a0 + 2.0 * PI * k / n;
        out.push([cx + r * a.cos(), cy + r * a.sin()]);
        k += 1.0;
    }
    out
}
fn ngon0(n: f64, r: f64, a0: f64) -> Vec<Pt> { ngon(n, r, a0, 0.0, 0.0) }

// ---------------------------------------------------------------- stems
/// straight runs between points about a quarter of the stem apart (at most 1.2 cm,
/// at least 0.3), as a ribbon with mitered corners; thin ones as a stroke
pub fn stem(pts: &[Pt], w0: f64, w1: f64, c: Option<Color>, op: Option<f64>, tip: Option<f64>) {
    let (s, big_l) = ink::arclen(pts);
    let seg = 0.3f64.max(1.2f64.min(big_l / 4.0));
    let (mut keep, mut ks, mut nextat) = (vec![pts[0]], vec![0.0], seg);
    for i in 1..pts.len() - 1 {
        if s[i] >= nextat {
            keep.push(pts[i]);
            ks.push(s[i]);
            nextat = s[i] + seg;
        }
    }
    keep.push(pts[pts.len() - 1]);
    ks.push(big_l);
    let wbp = (w0 + w1) / 2.0 * ink::unit() * ink::CM;
    if wbp < 0.7 || keep.len() < 2 {
        ink::poly(&keep, false);
        stroke(c.unwrap_or_else(|| tone(0.07)), 0.15f64.max(wbp), op);
        return;
    }
    let f = tip.map(|t| 1f64.min(t / big_l.max(1e-9)));
    let width = |t: f64| lerp(w0, w1, t) * match f {
        Some(f) if t > 1.0 - f => lerp(1.0, 0.15, (t - 1.0 + f) / f),
        _ => 1.0,
    };
    let nrm = |a: Pt, b: Pt| -> (f64, f64) {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let l = (dx * dx + dy * dy).sqrt();
        if l < 1e-9 { return (0.0, 1.0) }
        (-dy / l, dx / l)
    };
    let n = keep.len();
    let (mut left, mut right) = (vec![], vec![]);
    for i in 0..n {
        let a = if i > 0 { Some(nrm(keep[i - 1], keep[i])) } else { None };
        let b = if i + 1 < n { Some(nrm(keep[i], keep[i + 1])) } else { None };
        let a = a.or(b).unwrap();
        let b = b.unwrap_or(a);
        let (ax, ay, bx, by) = (a.0, a.1, b.0, b.1);
        let (mut nx, mut ny) = (ax + bx, ay + by);
        let m = (nx * nx + ny * ny).sqrt();
        if m < 1e-9 { nx = ax; ny = ay } else { nx /= m; ny /= m }
        let h = width(ks[i] / big_l) / 2.0 / 0.5f64.max(nx * ax + ny * ay);
        left.push([keep[i][0] + nx * h, keep[i][1] + ny * h]);
        right.push([keep[i][0] - nx * h, keep[i][1] - ny * h]);
    }
    for i in (0..right.len()).rev() { left.push(right[i]) }
    facet(&left, c.unwrap_or_else(|| tone(0.07)), op);
}

// ---------------------------------------------------------------- foliage
/// A leaf pointing +y, length 1: a kite, or with depth > 0 a polygon with triangular
/// lobes, folded along a straight midrib into a light and a dark half, with straight
/// white veins. Opts as flora.leaf: w, depth, lobes, bend, tone.
pub fn leaf(rng: &Rng, o: &Opts) {
    let w = o.w.unwrap_or_else(|| rng.range(0.2, 0.27));
    let bend = 0.5 * o.bend.unwrap_or_else(|| rng.sym(0.25));
    let depth = o.depth.unwrap_or_else(|| rng.range(0.3, 0.7));
    let t0 = o.tone.unwrap_or_else(|| rng.range(0.26, 0.38));
    let lobes = if depth > 0.0 { o.lobes.unwrap_or_else(|| rng.int(2.0, 4.0)) } else { 0.0 };
    let half = |v: f64| w * 2.5 * v.powf(0.6) * (1.0 - v);
    let side = |sg: f64| -> (Vec<Pt>, Vec<(Pt, f64)>) {
        let (mut pts, mut tips) = (vec![], vec![]);
        if lobes == 0.0 {
            pts.push([bend * 0.38 + sg * half(0.38) * 1.15, 0.38]);
            tips.push((pts[0], 0.3));
        } else {
            let mut k = 1.0;
            while k <= lobes {
                let (v1, v2) = (0.1 + 0.78 * (k - 0.35) / lobes, 0.1 + 0.78 * k / lobes);
                let p = [bend * v1 + sg * half(v1) * 1.2, v1 + 0.07];
                pts.push(p);
                pts.push([bend * v2 + sg * half(v2) * (1.0 - 0.7 * depth), v2]);
                tips.push((p, v1 - 0.06));
                k += 1.0;
            }
        }
        (pts, tips)
    };
    let tip = [bend, 1.0];
    let halfpoly = |pts: &[Pt]| -> Vec<Pt> {
        let mut out = vec![[0.0, 0.0]];
        out.extend_from_slice(pts);
        out.push(tip);
        out
    };
    let (lp, lt) = side(-1.0);
    let (rp, rt) = side(1.0);
    let dark = if rng.chance(0.5) { 1.0 } else { -1.0 };
    facet(&halfpoly(&lp), tone(t0 + 0.1 * dark), None);
    facet(&halfpoly(&rp), tone(t0 - 0.1 * dark), None);
    if fine(None) {
        ink::moveto(0.0, 0.0);
        ink::lineto(tip[0], tip[1]);
        for tp in [&lt, &rt] {
            for (p, v) in tp {
                ink::moveto(bend * v, *v);
                ink::lineto(lerp(bend * v, p[0], 0.85), lerp(*v, p[1], 0.85));
            }
        }
        if lobes == 0.0 {
            for v in [0.55, 0.72] {
                for sg in [-1.0, 1.0] {
                    let w = half(0.38) * 1.15 * (1.0 - v) / 0.62;
                    ink::moveto(bend * v, v);
                    ink::lineto(bend * (v + 0.1) + sg * w * 0.8, v + 0.1);
                }
            }
        }
        stroke(WHITE, 0.35, Some(0.85));
    }
}

/// a small leaf: a kite in two facets
pub fn leaflet(rng: &Rng, o: &Opts) {
    let w = o.w.unwrap_or_else(|| rng.range(0.16, 0.24));
    let bend = 0.5 * o.bend.unwrap_or_else(|| rng.sym(0.2));
    let t0 = o.tone.unwrap_or_else(|| rng.range(0.2, 0.34));
    let (tip, l, r) = ([bend, 1.0], [bend * 0.4 - w, 0.4], [bend * 0.4 + w, 0.4]);
    let dark = if rng.chance(0.5) { 1.0 } else { -1.0 };
    facet(&[[0.0, 0.0], l, tip], tone(t0 + 0.08 * dark), None);
    facet(&[[0.0, 0.0], r, tip], tone(t0 - 0.08 * dark), None);
    if fine(Some(0.3)) {
        ink::moveto(0.0, 0.0);
        ink::lineto(tip[0], tip[1]);
        stroke(WHITE, 0.3, Some(0.8));
    }
}

/// a petal (falling, or loose): a rhombus in two facets, length 1
pub fn petal(rng: &Rng, o: &Opts) {
    let w = o.w.unwrap_or_else(|| rng.range(0.22, 0.3));
    let t0 = o.tone.unwrap_or_else(|| rng.range(0.34, 0.42));
    facet(&[[0.0, 0.0], [w, 0.5], [0.0, 1.0]], tone(t0 - 0.06), None);
    facet(&[[0.0, 0.0], [0.0, 1.0], [-w, 0.5]], tone(t0 + 0.06), None);
}

// ---------------------------------------------------------------- flowers
/// the center: a pale polygon, its navy rim, a copper point
fn center(n: f64, r: f64, a0: f64) {
    facet(&ngon0(n, r, a0), WHITE, Some(0.92));
    ink::poly(&ngon0(n, r, a0), true);
    stroke(tone(0.06), 0.4, Some(0.85));
    ink::circle(0.0, 0.0, r * 0.3);
    ink::fill(ink::accent(-0.1), None, false);
}

/// A flower seen from the front, radius s (cm): five, six or eight rhombus petals, each
/// split along its axis into a light and a dark facet, darker triangles between them; or
/// a star polygon over a smaller one turned half a step.
pub fn flower(rng: &Rng, s: f64, o: &Opts) {
    ink::push();
    ink::scale(s, None);
    ink::rotate(rng.range(0.0, 360.0));
    let n = o.petals.unwrap_or_else(|| [5.0, 6.0, 8.0][rng.int(1.0, 3.0) as usize - 1]);
    let t0 = o.tone.unwrap_or_else(|| if rng.chance(0.25) { rng.range(0.18, 0.26) } else { rng.range(0.3, 0.42) });
    if o.style.unwrap_or_else(|| rng.int(1.0, 2.0)) == 1.0 {
        let w = 0.55 * (PI / n).sin() * 0.95;
        let mut k = 0.0;
        while k <= n - 1.0 {
            facet(&turn(&[[0.0, 0.1], [w * 0.5, 0.45], [0.0, 0.7], [-w * 0.5, 0.45]], 2.0 * PI * k / n + PI / n), tone(t0 - 0.16), None);
            k += 1.0;
        }
        k = 0.0;
        while k <= n - 1.0 {
            let p = turn(&[[0.0, 0.12], [w, 0.55], [0.0, 1.0], [-w, 0.55]], 2.0 * PI * k / n);
            facet(&[p[0], p[1], p[2]], tone(t0 - 0.07), None);
            facet(&[p[0], p[2], p[3]], tone(t0 + 0.09), None);
            k += 1.0;
        }
    } else {
        for layer in 1..=2 {
            let (rr, r, a0) = if layer == 1 { (1.0, 0.42, 0.0) } else { (0.66, 0.3, PI / n) };
            let mut k = 0.0;
            while k <= n - 1.0 {
                let a = a0 + 2.0 * PI * k / n;
                let o_ = [rr * (a + PI / 2.0).cos(), rr * (a + PI / 2.0).sin()];
                let l = [r * (a + PI / 2.0 - PI / n).cos(), r * (a + PI / 2.0 - PI / n).sin()];
                let rr_ = [r * (a + PI / 2.0 + PI / n).cos(), r * (a + PI / 2.0 + PI / n).sin()];
                let t = if layer == 1 { t0 } else { t0 + 0.22 };
                facet(&[[0.0, 0.0], l, o_], tone(t - 0.07), None);
                facet(&[[0.0, 0.0], o_, rr_], tone(t + 0.07), None);
                k += 1.0;
            }
        }
    }
    center(n, 0.17, PI / 2.0);
    ink::pop();
}

/// A star lily seen at a tilt: six triangular petals squashed along y, those behind
/// darker; straight stamens. Center at the origin, faces +y; s = petal length.
pub fn lily(rng: &Rng, s: f64, o: &Opts) {
    ink::push();
    ink::scale(s, None);
    let tilt = o.tilt.unwrap_or_else(|| rng.range(0.45, 0.62));
    let t0 = o.tone.unwrap_or_else(|| rng.range(0.3, 0.38));
    let spin = rng.range(0.0, 60.0);
    let mut order: Vec<f64> = (0..6).map(|k| spin + k as f64 * 60.0).collect();
    order.sort_by(|a, b| (b * RAD).sin().partial_cmp(&(a * RAD).sin()).unwrap());
    for a in order {
        let back = (a * RAD).sin();
        let mut p = turn(&[[0.0, 0.0], [0.14, 0.38], [0.0, 1.0], [-0.14, 0.38]], (a - 90.0) * RAD);
        for q in p.iter_mut() { *q = [q[0], q[1] * tilt] }
        let t = t0 - 0.08 * back;
        facet(&[p[0], p[1], p[2]], tone(t - 0.06), None);
        facet(&[p[0], p[2], p[3]], tone(t + 0.08), None);
    }
    if fine(Some(0.3)) {
        for k in 1..=5 {
            let a = ((k - 3) * 15) as f64 * RAD;
            ink::moveto(0.0, 0.0);
            ink::lineto(0.42 * a.sin(), 0.42 * a.cos());
        }
        stroke(tone(0.1), 0.3, Some(0.85));
        for k in 1..=5 {
            let a = ((k - 3) * 15) as f64 * RAD;
            let (x, y) = (0.42 * a.sin(), 0.42 * a.cos());
            ink::poly(&[[x, y - 0.03], [x + 0.02, y], [x, y + 0.03], [x - 0.02, y]], true);
        }
        ink::fill(tone(0.05), None, false);
    }
    ink::circle(0.0, 0.0, 0.035);
    ink::fill(ink::accent(-0.2), None, false);
    ink::pop();
}

/// a small flower: five kites, a pale center, a navy eye. r = radius (cm)
pub fn blossom(rng: &Rng, r: f64, o: &Opts) {
    ink::push();
    ink::scale(r, None);
    ink::rotate(rng.range(0.0, 72.0));
    let t0 = o.tone.unwrap_or_else(|| rng.range(0.3, 0.45));
    for k in 0..5 {
        let p = turn(&[[0.0, 0.1], [0.34, 0.6], [0.0, 1.0], [-0.34, 0.6]], (k * 2) as f64 * PI / 5.0);
        facet(&[p[0], p[1], p[2]], tone(t0 - 0.06), None);
        facet(&[p[0], p[2], p[3]], tone(t0 + 0.06), None);
    }
    facet(&ngon0(5.0, 0.24, PI / 2.0), WHITE, Some(0.9));
    facet(&ngon0(5.0, 0.1, PI / 2.0), o.eye.unwrap_or_else(|| tone(0.08)), None);
    ink::pop();
}

/// a floret a few millimeters across: a five-pointed star, a dark eye
pub fn floret(rng: &Rng, r: f64, o: &Opts) {
    let ph = rng.range(0.0, 2.0 * PI);
    let mut pts = vec![];
    for k in 0..10 {
        let a = ph + k as f64 * PI / 5.0;
        let d = if k % 2 == 0 { r } else { r * 0.42 };
        pts.push([d * a.cos(), d * a.sin()]);
    }
    facet(&pts, tone(o.tone.unwrap_or_else(|| rng.range(0.22, 0.42))), None);
    facet(&ngon0(5.0, 0.2 * r, ph), o.eye.unwrap_or_else(|| tone(0.05)), None);
}

/// a closed bud: a long rhombus in two facets, three triangular sepals. +y, length s
pub fn bud(rng: &Rng, s: f64, o: &Opts) {
    ink::push();
    ink::scale(s, None);
    let bend = rng.sym(0.08);
    let t0 = o.tone.unwrap_or_else(|| rng.range(0.3, 0.42));
    facet(&[[0.0, 0.0], [0.24, 0.42], [bend, 1.0]], tone(t0 - 0.07), None);
    facet(&[[0.0, 0.0], [bend, 1.0], [-0.24, 0.42]], tone(t0 + 0.08), None);
    for k in -1..=1 {
        facet(&turn(&[[-0.05, -0.02], [0.05, -0.02], [0.0, if k == 0 { 0.24 } else { 0.3 }]], (-k * 28) as f64 * RAD), tone(0.12), None);
    }
    ink::pop();
}

/// a bellflower hanging from the origin along -y: a flared trapezoid with a zigzag rim,
/// in three facets, a calyx of triangles. s = length
pub fn bell(rng: &Rng, s: f64, o: &Opts) {
    ink::push();
    ink::scale(s, None);
    ink::rotate(rng.sym(8.0));
    let t0 = o.tone.unwrap_or_else(|| rng.range(0.3, 0.4));
    let rim = [[0.45, -0.98], [0.3, -0.86], [0.15, -1.0], [0.0, -0.88], [-0.15, -1.0], [-0.3, -0.86], [-0.45, -0.98]];
    let mut body = vec![[0.12, -0.02]];
    body.extend_from_slice(&rim);
    body.push([-0.12, -0.02]);
    facet(&body, tone(t0), None);
    facet(&[[0.12, -0.02], [0.45, -0.98], [0.3, -0.86], [0.15, -1.0], [0.06, -0.02]], tone(t0 - 0.1), None);
    facet(&[[-0.12, -0.02], [-0.06, -0.02], [-0.15, -1.0], [-0.3, -0.86], [-0.45, -0.98]], tone(t0 + 0.12), None);
    for k in -1..=1 {
        facet(&turn(&[[-0.05, 0.02], [0.05, 0.02], [0.0, 0.28]], (180 + k * 40) as f64 * RAD), tone(0.1), None);
    }
    ink::pop();
}

// ---------------------------------------------------------------- sprays
/// a forget-me-not cyme: a stalk that turns more at each step, straight between them,
/// florets crowding along it and shrinking into buds. Starts heading +y.
#[allow(clippy::too_many_arguments)]
fn curl(rng: &Rng, x: f64, y: f64, h0: f64, l: f64, side: f64, t0: f64, w: f64) -> Vec<Pt> {
    let n = 9.0;
    let turnall = rng.range(170.0, 260.0);
    let (mut x, mut y) = (x, y);
    let mut pts = vec![[x, y]];
    let mut a = h0;
    for i in 1..=9 {
        let i = i as f64;
        a = a + side * turnall * ((i / n).powf(2.3) - ((i - 1.0) / n).powf(2.3));
        x = x + l / n * (a * RAD).cos();
        y = y + l / n * (a * RAD).sin();
        pts.push([x, y]);
    }
    ink::poly(&pts, false);
    stroke(tone(0.1), 0.2f64.max(w * ink::unit() * ink::CM), None);
    for i in 2..=pts.len() {
        let f = (i - 1) as f64 / n;
        let (p, q) = (pts[i - 2], pts[i - 1]);
        let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
        let len = (dx * dx + dy * dy).sqrt();
        let sg = if i % 2 == 0 { 1.0 } else { -1.0 };
        let r = lerp(0.11, 0.035, f);
        let (cx, cy) = (q[0] - dy / len * sg * (r + 0.02), q[1] + dx / len * sg * (r + 0.02));
        ink::push();
        ink::translate(cx, cy);
        if f > 0.75 {
            let g = ngon0(4.0, r * 0.7, rng.range(0.0, PI));
            facet(&g, tone(t0 - 0.06), None);
        } else {
            let g = ngon0(5.0, r, rng.range(0.0, 2.0 * PI));
            facet(&g, tone(t0 + rng.sym(0.08)), None);
            facet(&ngon0(5.0, r * 0.3, 0.0), tone(0.05), None);
        }
        ink::pop();
    }
    pts
}

pub fn cyme(rng: &Rng, l: f64, o: &Opts) {
    let side = o.side.unwrap_or_else(|| if rng.chance(0.5) { 1.0 } else { -1.0 });
    let t0 = rng.range(0.22, 0.36);
    let h0 = 90.0 + rng.sym(10.0);
    let pts = curl(rng, 0.0, 0.0, h0, l, side, t0, 0.024);
    let forked = o.fork != Some(false) && rng.chance(if o.fork == Some(true) { 1.0 } else { 0.6 });
    if forked {
        let i = rng.int(3.0, 4.0) as usize;
        let a = (pts[i][1] - pts[i - 1][1]).atan2(pts[i][0] - pts[i - 1][0]) / RAD;
        let h = a - side * rng.range(30.0, 50.0);
        let len = l * rng.range(0.45, 0.65);
        curl(rng, pts[i - 1][0], pts[i - 1][1], h, len, -side, t0 + 0.04, 0.016);
    }
}

fn branch(rng: &Rng, t0: f64, depth: f64, x: f64, y: f64, a: f64, len: f64, d: f64, w: f64) {
    let (ex, ey) = (x + len * (a * RAD).cos(), y + len * (a * RAD).sin());
    ink::moveto(x, y);
    ink::lineto(ex, ey);
    stroke(tone(0.1), 0.2f64.max(w * ink::unit() * ink::CM), None);
    if d == 0.0 {
        ink::push();
        ink::translate(ex, ey);
        if rng.chance(0.7) {
            let r = rng.range(0.045, 0.075);
            floret(rng, r, &Opts { tone: Some(t0 + rng.sym(0.08)), ..Opts::default() });
        } else {
            let g = ngon0(4.0, rng.range(0.02, 0.03), a * RAD);
            facet(&g, tone(t0 - 0.05), None);
        }
        ink::pop();
        return;
    }
    if d >= depth - 1.0 && rng.chance(0.7) {
        let sg = if rng.chance(0.5) { 1.0 } else { -1.0 };
        ink::push();
        ink::translate(ex, ey);
        ink::rotate(a - 90.0 + sg * rng.range(40.0, 70.0));
        ink::scale(len * rng.range(0.5, 0.8), None);
        leaflet(rng, &Opts { w: Some(0.14), ..Opts::default() });
        ink::pop();
    }
    let kids = if rng.chance(0.25) { 3.0 } else { 2.0 };
    let mut k = 1.0;
    while k <= kids {
        let da = if kids == 2.0 {
            (if k == 1.0 { -1.0 } else { 1.0 }) * rng.range(18.0, 34.0)
        } else {
            (k - 2.0) * rng.range(24.0, 36.0)
        };
        let nl = len * rng.range(0.58, 0.74);
        branch(rng, t0, depth, ex, ey, a + da, nl, d - 1.0, w * 0.72);
        k += 1.0;
    }
}

/// a branching sprig, straight between the forks; star florets and diamond buds at the
/// tips, kites at the lower forks. Starts heading +y, length L.
pub fn sprig(rng: &Rng, l: f64, o: &Opts) {
    let depth = o.depth.unwrap_or(4.0);
    let t0 = rng.range(0.22, 0.4);
    let a = 90.0 + rng.sym(8.0);
    branch(rng, t0, depth, 0.0, 0.0, a, l * 0.36, depth, 0.024);
}

/// an umbel: straight rays fanning upward, each ending in a little umbel of star
/// florets; triangular bracts at the base. R = reach (cm)
pub fn umbel(rng: &Rng, r: f64, _o: &Opts) {
    let nr = rng.int(9.0, 13.0);
    let mut k = 1.0;
    while k <= nr {
        let a = (90.0 + (k - (nr + 1.0) / 2.0) * (110.0 / nr) + rng.sym(4.0)) * RAD;
        let l = r * rng.range(0.78, 1.0) * (1.0 - 0.15 * (a / RAD - 90.0).abs() / 55.0);
        let (ex, ey) = (l * a.cos(), l * a.sin());
        ink::moveto(0.0, 0.0);
        ink::lineto(ex, ey);
        stroke(tone(0.12), 0.25, Some(0.9));
        let ns = rng.int(5.0, 7.0);
        let mut j = 1.0;
        while j <= ns {
            let b = a + ((j - (ns + 1.0) / 2.0) * 16.0 + rng.sym(5.0)) * RAD;
            let l = r * rng.range(0.12, 0.2);
            ink::moveto(ex, ey);
            ink::lineto(ex + l * b.cos(), ey + l * b.sin());
            stroke(tone(0.14), 0.18, Some(0.85));
            ink::push();
            ink::translate(ex + l * b.cos(), ey + l * b.sin());
            let fr = r * rng.range(0.035, 0.05);
            floret(rng, fr, &Opts { tone: Some(rng.range(0.25, 0.42)), ..Opts::default() });
            ink::pop();
            j += 1.0;
        }
        k += 1.0;
    }
    for k in -1..=1 {
        facet(&turn(&[[-0.03 * r, 0.0], [0.03 * r, 0.0], [0.0, -0.2 * r]], (k * 40) as f64 * RAD), tone(0.15), None);
    }
}

/// a square spiral: straight runs, each turning left a right angle and shorter by the
/// golden ratio. Starts at the origin; s = size (cm)
pub fn tendril(_rng: &Rng, s: f64) {
    let (mut x, mut y, mut a, mut l) = (0.0, 0.0, 90.0, s * 1.15);
    let mut pts = vec![[0.0, 0.0]];
    for _ in 0..9 {
        x = x + l * (a * RAD).cos();
        y = y + l * (a * RAD).sin();
        pts.push([x, y]);
        a += 90.0;
        l /= phi();
    }
    ink::poly(&pts, false);
    stroke(tone(0.12), 0.45, None);
}

/// a rosette accent: the star polygon {8/3} in a copper hairline
pub fn rosette(_rng: &Rng, r: f64, o: &Opts) {
    let c = o.color.unwrap_or_else(|| ink::accent(0.0));
    let mut pts = vec![];
    for k in 0..8 {
        let a = ((k * 3 * 45) as f64 + 22.5) * RAD;
        pts.push([1.3 * r * a.cos(), 1.3 * r * a.sin()]);
    }
    ink::poly(&pts, true);
    stroke(c, 0.22, Some(0.85));
    ink::circle(0.0, 0.0, r * 0.14);
    ink::fill(c, None, false);
}

/// the header fleuron: a copper kite leaf, tip down-left, its stem straight out to the
/// right where the head rule continues. Returns the stem's end in page cm.
pub fn hedera(_rng: &Rng, s: f64, o: &Opts) -> (f64, f64) {
    let c = o.color.unwrap_or_else(|| ink::accent(0.0));
    ink::push();
    ink::scale(s, None);
    let p = turn(&[[0.0, 0.0], [0.42, -0.45], [0.0, -1.25], [-0.42, -0.45]], -38.0 * RAD);
    facet(&p, c, None);
    ink::moveto(p[0][0], p[0][1]);
    ink::lineto(p[2][0], p[2][1]);
    stroke(PAPER, 0.35, None);
    ink::poly(&[[0.0, 0.0], [0.3, 0.36], [0.9, 0.36], [2.0, 0.28]], false);
    stroke(c, 0.4, None);
    let (rx, ry) = ink::to_page(2.0, 0.28);
    ink::pop();
    (rx, ry)
}

// ---------------------------------------------------------------- visitors
/// a fan of facets from `root` to the outline's consecutive corners, tones alternating
fn fan(root: Pt, pts: &[Pt], t0: f64, dt: f64) {
    for i in 1..pts.len() {
        facet(&[root, pts[i - 1], pts[i]], tone(t0 + if i % 2 == 0 { dt } else { -dt }), None);
    }
}

/// a butterfly from above, wings of straight-edged facets; s = wingspan, o.open narrows
/// the wings (a wingbeat). Head toward +y, center at the origin.
pub fn butterfly(rng: &Rng, s: f64, o: &Opts) {
    let open = o.open.unwrap_or(1.0);
    ink::push();
    ink::scale(s / 2.0, None);
    let t0 = o.tone.unwrap_or_else(|| rng.range(0.28, 0.4));
    let fw = [[0.05, 0.1], [0.3, 0.48], [0.68, 0.7], [0.98, 0.62], [0.9, 0.3], [0.55, 0.06], [0.12, -0.02]];
    let hw = [[0.06, -0.02], [0.5, -0.04], [0.74, -0.3], [0.6, -0.68], [0.3, -0.7], [0.08, -0.36]];
    for sx in [-1.0, 1.0] {
        let side = |w: &[Pt]| -> Vec<Pt> { w.iter().map(|p| [sx * p[0] * open, p[1]]).collect() };
        fan([sx * 0.03 * open, -0.03], &side(&hw), t0 + 0.08, 0.06);
        fan([sx * 0.03 * open, 0.04], &side(&fw), t0, 0.07);
        if fine(Some(0.3)) {
            facet(&[[sx * 0.62 * open, 0.42], [sx * 0.7 * open, 0.48], [sx * 0.62 * open, 0.54], [sx * 0.54 * open, 0.48]], ink::accent(0.0), None);
        }
    }
    facet(&[[0.0, 0.3], [0.05, 0.12], [0.04, -0.3], [0.0, -0.6], [-0.04, -0.3], [-0.05, 0.12]], tone(0.05), None);
    ink::poly(&[[-0.22, 0.86], [-0.03, 0.32], [0.03, 0.32], [0.22, 0.86]], false);
    stroke(tone(0.08), 0.35, None);
    ink::pop();
}

/// a bee from above: a faceted hexagonal abdomen with bands, a dark thorax, pale rhombic
/// wings. Head toward +y; s = body length
pub fn bee(_rng: &Rng, s: f64, _o: &Opts) {
    ink::push();
    ink::scale(s, None);
    let ab = [[0.0, 0.08], [0.17, -0.02], [0.17, -0.34], [0.0, -0.52], [-0.17, -0.34], [-0.17, -0.02]];
    facet(&ab, tone(0.4), None);
    for k in 0..3 {
        let y = -0.08 - 0.13 * k as f64;
        facet(&[[-0.17, y + 0.03], [0.17, y + 0.03], [0.17, y - 0.03], [-0.17, y - 0.03]], tone(0.05), None);
    }
    facet(&ngon(6.0, 0.13, PI / 6.0, 0.0, 0.16), tone(0.06), None);
    facet(&ngon(5.0, 0.08, PI / 2.0, 0.0, 0.34), tone(0.04), None);
    for sx in [-1.0, 1.0] {
        let w = [[sx * 0.06, 0.18], [sx * 0.3, 0.32], [sx * 0.5, 0.2], [sx * 0.26, 0.06]];
        facet(&w, tone(0.86), Some(0.6));
        ink::poly(&w, true);
        stroke(tone(0.2), 0.3, Some(0.8));
    }
    ink::pop();
}

/// a snail crawling up +y, foot on the y axis, shell on the +x side: the shell a golden
/// rectangle cut into squares, a straight-edged spiral through them. s = shell diameter
pub fn snail(_rng: &Rng, s: f64, _o: &Opts) {
    ink::push();
    ink::scale(s, None);
    facet(&[[0.0, -0.78], [0.12, -0.45], [0.17, 0.3], [0.24, 0.5], [0.22, 0.66], [0.12, 0.7], [0.04, 0.58], [-0.01, -0.3]], tone(0.55), None);
    ink::poly(&[[0.17, 0.66], [0.36, 1.02]], false);
    ink::poly(&[[0.1, 0.7], [0.18, 1.04]], false);
    stroke(tone(0.12), 0.4, None);
    let (mut x, mut y, mut w, mut h) = (0.36 - 0.27, -0.42, 0.52, 0.84);
    let mut spiral: Vec<Pt> = vec![];
    for i in 0..8 {
        let sq: [f64; 3];
        match i % 4 {
            0 => { sq = [x, y + h - w, w]; spiral.push([x, y + h]); h = h - w; }
            1 => { sq = [x + w - h, y, h]; spiral.push([x + w, y + h]); w = w - h; }
            2 => { sq = [x, y, w]; spiral.push([x + w, y]); let (ny, nh) = (y + w, h - w); y = ny; h = nh; }
            _ => { sq = [x, y, h]; spiral.push([x, y]); let (nx, nw) = (x + h, w - h); x = nx; w = nw; }
        }
        facet(&[[sq[0], sq[1]], [sq[0] + sq[2], sq[1]], [sq[0] + sq[2], sq[1] + sq[2]], [sq[0], sq[1] + sq[2]]],
            tone(0.22 + 0.08 * (i % 3) as f64), None);
    }
    ink::poly(&spiral, false);
    stroke(WHITE, 0.35, Some(0.85));
    ink::pop();
}

/// a perched bird facing +x (o.facing = -1: -x), feet at the origin, all facets: a fan
/// from the middle of the body, the breast lighter; s = tail to beak
pub fn bird(rng: &Rng, s: f64, o: &Opts) {
    ink::push();
    ink::scale(o.facing.unwrap_or(1.0) * s, Some(s));
    let t0 = o.tone.unwrap_or_else(|| rng.range(0.3, 0.38));
    for k in 1..=3 {
        let k = k as f64;
        facet(&[[-0.24, 0.3], [-0.62 + 0.04 * k, 0.04 - 0.05 * k], [-0.56 + 0.04 * k, 0.0 - 0.05 * k]], tone(0.12 + 0.06 * k), None);
    }
    let body = [[0.37, 0.56], [0.28, 0.67], [0.12, 0.64], [-0.08, 0.52], [-0.3, 0.32], [-0.24, 0.2], [0.04, 0.12], [0.24, 0.24], [0.36, 0.42], [0.37, 0.56]];
    for i in 1..body.len() {
        let front = body[i - 1][0] > 0.15 && body[i - 1][1] < 0.5;
        let t = if front { 0.72 } else { t0 + if i % 2 == 0 { 0.05 } else { -0.05 } };
        facet(&[[0.08, 0.4], body[i - 1], body[i]], tone(t), None);
    }
    facet(&[[0.16, 0.46], [-0.12, 0.46], [-0.44, 0.22], [-0.1, 0.28]], tone(t0 - 0.12), None);
    facet(&[[0.36, 0.585], [0.47, 0.545], [0.36, 0.51]], tone(0.05), None);
    ink::circle(0.285, 0.585, 0.025);
    ink::fill(tone(0.03), None, false);
    ink::poly(&[[0.02, 0.14], [0.0, 0.0]], false);
    ink::poly(&[[0.1, 0.14], [0.08, 0.0]], false);
    stroke(tone(0.06), 0.45, None);
    ink::pop();
}

// ---------------------------------------------------------------- new motifs
/// a crystal: a Platonic solid flat-shaded like a cut stone, white edges; r = radius
pub fn crystal(rng: &Rng, r: f64, o: &Opts) {
    let i = o.k.unwrap_or_else(|| rng.int(1.0, 5.0)) as usize;
    let solids = figures::solids();
    if i < 1 || i > solids.len() { return }
    let v = &solids[i - 1].v;
    let h = figures::hull_of(v);
    let rot = figures::rotation(rng.range(0.0, 360.0), rng.range(15.0, 35.0));
    let k = r * 0.95 / h.radius;
    let p: Vec<Pt> = v.iter().map(|q| { let (x, y, _) = rot(q[0], q[1], q[2]); [x * k, y * k] }).collect();
    let t0 = rng.range(0.2, 0.35);
    for f in &h.faces {
        let (nx, ny, nz) = rot(f.n[0], f.n[1], f.n[2]);
        if nz > 1e-6 {
            let pts: Vec<Pt> = f.v.iter().map(|&m| p[m]).collect();
            let lit = 0f64.max(-0.45 * nx + 0.6 * ny + 0.66 * nz);
            facet(&pts, tone(lerp(t0, 0.85, lit)), None);
            ink::poly(&pts, true);
            stroke(WHITE, 0.3, Some(0.9));
        }
    }
}

// ---------------------------------------------------------------- the theme
/// `M.flora`
pub fn flora() -> Vec<(&'static str, Prim)> {
    vec![
        ("leaf", Prim::Scaled(leaf)), ("leaflet", Prim::Scaled(leaflet)), ("petal", Prim::Scaled(petal)),
        ("flower", Prim::Sized(flower)), ("lily", Prim::Sized(lily)), ("blossom", Prim::Sized(blossom)),
        ("floret", Prim::Sized(floret)), ("bud", Prim::Sized(bud)), ("bell", Prim::Sized(bell)),
        ("cyme", Prim::Sized(cyme)), ("sprig", Prim::Sized(sprig)), ("umbel", Prim::Sized(umbel)),
        ("tendril", Prim::Sized(|r, s, _| tendril(r, s))), ("seedhead", Prim::Sized(figures::phyllotaxis)),
        ("rosette", Prim::Sized(rosette)), ("hedera", Prim::Sized(|r, s, o| { hedera(r, s, o); })),
        ("butterfly", Prim::Sized(butterfly)), ("bee", Prim::Sized(bee)), ("snail", Prim::Sized(snail)), ("bird", Prim::Sized(bird)),
        ("crystal", Prim::Sized(crystal)),
    ]
}

fn spray(rng: &Rng, s: f64, n0: f64, n1: f64, spread: f64, f: &dyn Fn(f64)) {
    let n = rng.int(n0, n1);
    let mut k = 1.0;
    while k <= n {
        let a = ((k - (n + 1.0) / 2.0) * spread + rng.sym(10.0)) * RAD;
        let d = s * rng.range(0.45, 0.8) * (if k == 1.0 { 1.0 } else { rng.range(0.85, 1.1) });
        let (x, y) = (d * a.sin(), d * a.cos());
        ink::moveto(0.0, 0.0);
        ink::lineto(x, y);
        stroke(tone(0.1), 0.35, None);
        ink::push();
        ink::translate(x, y);
        f(a);
        ink::pop();
        k += 1.0;
    }
}

/// `M.motifs`: the sprays of florilegium-grow.lua on straight stalks, and the crystal
pub fn motifs(mut rows: Vec<Motif>, _f: &Flora) -> Vec<Motif> {
    for m in rows.iter_mut() {
        match m.name.as_str() {
            "blossoms" => m.draw = Rc::new(|f, rng, s, _| {
                spray(rng, s, 2.0, 4.0, 38.0, &|_| { let r = s * rng.range(0.38, 0.5); f.sized("blossom", rng, r, &Opts::default()) });
                if rng.chance(0.6) {
                    ink::push();
                    ink::rotate(rng.sym(50.0));
                    ink::scale(0.5 * s, None);
                    f.sized("bud", rng, 1.0, &Opts::default());
                    ink::pop();
                }
            }),
            "budding" => m.draw = Rc::new(|f, rng, s, _| {
                spray(rng, s, 2.0, 4.0, 34.0, &|a| {
                    ink::rotate(-a / RAD * 0.6);
                    let r = s * rng.range(0.45, 0.6);
                    f.sized("bud", rng, r, &Opts::default());
                });
            }),
            "bells" => m.draw = Rc::new(|f, rng, s, _| {
                f.sized("bell", rng, s, &Opts::default());
                ink::moveto(0.0, 0.0);
                ink::lineto(0.3 * s, 0.2 * s);
                ink::lineto(0.5 * s, 0.1 * s);
                stroke(tone(0.1), 0.35, None);
                ink::push();
                ink::translate(0.5 * s, 0.1 * s);
                f.sized("bell", rng, 0.75 * s, &Opts::default());
                ink::pop();
            }),
            _ => {}
        }
    }
    let mut m = Motif::new("crystal", 1.2, 0.45, 0.8, 2.0, L_BLOOM, |s| 1.0 * s, |_| 0.0,
        Rc::new(|f, rng, s, _| f.sized("crystal", rng, s, &Opts::default())));
    m.hard = true;
    m.upright = Some(0.4);
    rows.push(m);
    rows
}

fn theme_flora(f: &mut Flora) {
    for (name, p) in flora() {
        f.set(name, match p {
            Prim::Scaled(g) => GPrim::Scaled(Rc::new(move |r, o| g(r, o))),
            Prim::Sized(g) => GPrim::Sized(Rc::new(move |r, s, o| g(r, s, o))),
        });
    }
    let sf: StemFn = Rc::new(|pts, w0, w1, c, op, tip| stem(pts, w0, w1, c, op, tip));
    f.set_stem(sf);
}

pub fn theme_def() -> ThemeDef {
    ThemeDef { flora: theme_flora, motifs: Some(motifs), page: None, named: None }
}

