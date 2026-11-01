// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! Port of figures.lua: mathematical figures for the math and geometric themes.
//!
//! Parameters are `Vec<f64>`/`&[f64]` throughout. Where the Lua mixes in other types they are
//! encoded as numbers: booleans are 1.0/0.0 (a missing trailing entry is nil), and the index
//! of a solid or a knot in `p[0]` stays 1-based as in the Lua (`solids()` itself is 0-based).
//! The picks only ever hold Lua integers or floats, so `%d` and `%s` captions print integral
//! values as integers.
//! Family values are `Vec<f64>`: a number is `[v]`, a pair `[a, b]`, and the truncation
//! families' `{solid, s, label}` is `[solid, s, kind]` (see `truncation`).
use crate::fmt;
use crate::ink::{self, Color, Rng};
use crate::flora::Opts;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::rc::Rc;
use std::sync::OnceLock;

type Pt = [f64; 2];

const RAD: f64 = PI / 180.0;
fn phi() -> f64 { (1.0 + 5f64.sqrt()) / 2.0 }
fn golden() -> f64 { PI * (3.0 - 5f64.sqrt()) }
fn lerp(a: f64, b: f64, t: f64) -> f64 { a + (b - a) * t }
fn ghost() -> bool { ink::get_lift() > 0.3 }
fn gcd(mut a: i64, mut b: i64) -> i64 { while b != 0 { let t = a % b; a = b; b = t } a }
fn ink_line() -> Color { ink::tone(0.08) }
fn tone(t: f64) -> Color { ink::tone(t) }
fn st(c: Color, w: f64) { ink::stroke(c, w, None, false, None) }
fn st1(c: Color, w: f64, op: f64) { ink::stroke(c, w, Some(op), false, None) }
fn fl(c: Color) { ink::fill(c, None, false) }
fn flo(c: Color, op: f64, mult: bool) { ink::fill(c, Some(op), mult) }
fn sm(pts: &[Pt], closed: bool) { ink::smooth(pts, closed, None) }
fn dd(out: &mut String, x: f64) { fmt::lua_int(out, x as i64) }
fn square(h: f64) -> [Pt; 4] { [[-h, -h], [h, -h], [h, h], [-h, h]] }

/// a small seed or floret: a hexagon of radius r, a corner along angle a (radians)
fn seed(x: f64, y: f64, r: f64, a: f64) {
    let pts: Vec<Pt> = (0..6).map(|k| { let b = a + k as f64 * PI / 3.0; [x + r * b.cos(), y + r * b.sin()] }).collect();
    ink::poly(&pts, true);
}

// ---------------------------------------------------------------- curves
fn nlobes(n: f64, d: f64) -> f64 { if (n * d) % 2.0 == 1.0 { n } else { 2.0 * n } }
fn lobe(k: f64, m: usize) -> Vec<Pt> {
    let h = PI / (2.0 * k);
    (0..m).map(|i| {
        let p = -h + 2.0 * h * i as f64 / m as f64;
        let r = (k * p).cos();
        [-r * p.sin(), r * p.cos()]
    }).collect()
}

// ---------------------------------------------------------------- the golden angle
/// Vogel's model of a sunflower head: {x, y, r/R, angle}
pub fn vogel(n: usize, r: f64, spin: f64) -> Vec<[f64; 4]> {
    (1..=n).map(|i| {
        let a = spin + i as f64 * golden();
        let q = ((i as f64 - 0.5) / n as f64).sqrt();
        [r * q * a.cos(), r * q * a.sin(), q, a]
    }).collect()
}

fn disc(n: usize, r: f64, spin: f64, t_in: f64, t_out: f64, gone: &[bool], size: Option<f64>) -> (Vec<[f64; 4]>, f64) {
    let f = vogel(n, r, spin);
    let c = r / (n as f64).sqrt() * size.unwrap_or(0.62);
    for band in 0..4 {
        for (i, p) in f.iter().enumerate() {
            if !gone.get(i).copied().unwrap_or(false) && (p[2] * 3.999).floor() == band as f64 {
                seed(p[0], p[1], c * (0.8 + 0.4 * p[2]), p[3]);
            }
        }
        fl(tone(lerp(t_in, t_out, band as f64 / 3.0)));
    }
    (f, c)
}

fn parastichies(f: &[[f64; 4]], steps: &[[usize; 2]]) {
    for st_ in steps {
        let (step, from) = (st_[0], st_[1]);
        for j in from..from + step {
            let mut pts: Vec<Pt> = vec![];
            let mut i = j;
            while i <= f.len() { pts.push([f[i - 1][0], f[i - 1][1]]); i += step }
            if pts.len() > 2 { sm(&pts, false) }
        }
    }
}

/// The disc gone to seed. `o.side` and `o.drift` are optional as in the Lua.
pub fn phyllotaxis(rng: &Rng, r: f64, o: &Opts) {
    ink::push();
    ink::scale(r, None);
    let fine = ink::unit() > 0.2 && !ghost();
    let n = if fine { 150 } else { 70 };
    let rr = 0.8;
    let spin = rng.range(0.0, 2.0 * PI);
    let f = vogel(n, rr, spin);
    let mut gone = vec![];
    for p in &f { gone.push(p[2] > 0.85 && rng.chance(0.3)) }
    ink::circle(0.0, 0.0, rr * 0.98);
    fl(tone(0.8));
    let (_, c) = disc(n, rr, spin, 0.03, 0.2, &gone, Some(0.5));
    if fine && ink::unit() > 0.9 {
        parastichies(&f, &[[21, 30], [34, 45]]);
        st1(ink::accent(0.0), 0.25, 0.85);
    }
    let side = match o.side { Some(s) => s, None => if rng.chance(0.5) { 1.0 } else { -1.0 } };
    let count = match o.drift { Some(d) => d, None => rng.int(2.0, 4.0) };
    let mut k = 1.0;
    while k <= count {
        let a = (90.0 - side * rng.range(20.0, 70.0)) * RAD;
        let d = rng.range(1.0, 1.5);
        seed(d * a.cos(), d * a.sin(), c * 1.3, rng.range(0.0, PI));
        k += 1.0;
    }
    fl(tone(0.1));
    ink::pop();
}

const MAURER: [[f64; 2]; 6] = [[2.0, 39.0], [6.0, 71.0], [5.0, 97.0], [4.0, 31.0], [7.0, 19.0], [3.0, 47.0]];

/// A Maurer rose; `k` = {n, d}, drawn from the rng when None.
pub fn maurer(rng: &Rng, r: f64, k: Option<[f64; 2]>) {
    let p = match k { Some(k) => k, None => MAURER[rng.int(1.0, MAURER.len() as f64) as usize - 1] };
    let (n, d) = (p[0], p[1]);
    ink::push();
    ink::scale(r, None);
    ink::rotate(rng.range(0.0, 360.0));
    let mut rose: Vec<Pt> = vec![];
    let nn = nlobes(n, 1.0);
    let out = lobe(n, 16);
    let mut j = 0.0;
    while j <= nn - 1.0 {
        ink::push();
        ink::rotate(90.0 / n - 90.0 + j * 360.0 / nn);
        sm(&out, true);
        ink::pop();
        j += 1.0;
    }
    flo(tone(0.62), 0.7, true);
    if !ghost() {
        let mut pts: Vec<Pt> = vec![];
        for i in 0..=360 {
            let th = i as f64 * d * RAD;
            let rr = (n * th).sin();
            pts.push([rr * th.cos(), rr * th.sin()]);
        }
        ink::poly(&pts, false);
        st1(tone(0.1), 0.12, 0.45);
        let period = if n % 2.0 == 1.0 { PI } else { 2.0 * PI };
        let mut i = 0.0;
        while i <= 8.0 * nn - 1.0 {
            let th = period * i / (8.0 * nn);
            let rr = (n * th).sin();
            rose.push([rr * th.cos(), rr * th.sin()]);
            i += 1.0;
        }
        sm(&rose, true);
        st1(tone(0.08), 0.3, 0.8);
        ink::circle(0.0, 0.0, 0.035);
        fl(ink::accent(0.0));
    }
    ink::pop();
}

// ---------------------------------------------------------------- the butterfly
/// Temple Fay's butterfly curve; `open` and `tone` are the Lua's `o.open`, `o.tone`.
pub fn fay(rng: &Rng, s: f64, open: Option<f64>, tone_: Option<f64>) {
    let open = open.unwrap_or(1.0);
    let fine = ink::unit() * s > 0.6 && !ghost();
    ink::push();
    ink::scale(s / 5.81, None);
    ink::translate(0.0, -0.66);
    ink::scale(open, Some(1.0));
    let t0 = match tone_ { Some(t) => t, None => rng.range(0.3, 0.42) };
    let curve = |m: usize, t_max: f64, slow: bool| -> Vec<Pt> {
        (0..m).map(|i| {
            let t = t_max * i as f64 / m as f64;
            let rho = t.cos().exp() - 2.0 * (4.0 * t).cos() - (if slow { (t / 12.0).sin().powf(5.0) } else { 0.0 });
            [t.sin() * rho, t.cos() * rho]
        }).collect()
    };
    let wings = curve(96, 2.0 * PI, false);
    sm(&wings, true);
    flo(tone(t0 + 0.12), 0.85, false);
    sm(&wings, true);
    ink::clip();
    crate::flora::mottle(rng, -3.0, -1.8, 3.0, 3.2, if fine { 10 } else { 4 }, 0.5, t0 - 0.2, t0 + 0.15, Some(if fine { 10 } else { 0 }));
    ink::unclip();
    if fine {
        sm(&curve(720, 12.0 * PI, true), true);
        st1(tone(0.06), 0.2, 0.75);
    }
    ink::push();
    ink::scale(1.0 / open, Some(1.0));
    let hb = |v: f64| 0.13 * (PI * v.powf(0.7)).sin();
    let (outline, _, _) = ink::blade(&|v| (0.0, -0.9 + 2.2 * v), 10, &hb, &hb, None);
    sm(&outline, true);
    fl(tone(0.05));
    ink::circle(0.0, 1.4, 0.16);
    fl(tone(0.04));
    for sx in [-1.0, 1.0] {
        ink::moveto(sx * 0.05, 1.5);
        ink::curveto(sx * 0.15, 2.0, sx * 0.4, 2.4, sx * 0.6, 2.6);
        st(tone(0.08), 0.35);
        ink::circle(sx * 0.6, 2.6, 0.06);
        fl(tone(0.06));
    }
    ink::pop();
    ink::pop();
}

// ---------------------------------------------------------------- 3-D
/// yaw about y, then pitch about x (degrees); the viewer looks along -z
pub fn rotation(yaw: f64, pitch: f64) -> impl Fn(f64, f64, f64) -> (f64, f64, f64) {
    let (ca, sa, cb, sb) = ((yaw * RAD).cos(), (yaw * RAD).sin(), (pitch * RAD).cos(), (pitch * RAD).sin());
    move |x, y, z| {
        let (x1, z1) = (ca * x + sa * z, -sa * x + ca * z);
        (x1, cb * y - sb * z1, sb * y + cb * z1)
    }
}

pub type V3 = [f64; 3];

fn signs(p: V3, out: &mut Vec<V3>) {
    let one = |c: f64| if c == 0.0 { vec![0.0] } else { vec![1.0, -1.0] };
    for sx in one(p[0]) { for sy in one(p[1]) { for sz in one(p[2]) {
        out.push([if sx == 0.0 { 0.0 } else { sx * p[0] }, if sy == 0.0 { 0.0 } else { sy * p[1] }, if sz == 0.0 { 0.0 } else { sz * p[2] }]);
    } } }
}
pub fn cyclic(list: &[V3]) -> Vec<V3> {
    let mut out = vec![];
    for p in list {
        signs(*p, &mut out);
        signs([p[2], p[0], p[1]], &mut out);
        signs([p[1], p[2], p[0]], &mut out);
    }
    out
}

pub struct Solid { pub name: &'static str, pub label: &'static str, pub v: Vec<V3> }

/// the Platonic solids (0-based here; the Lua's SOLIDS is 1-based)
pub fn solids() -> &'static [Solid] {
    static S: OnceLock<Vec<Solid>> = OnceLock::new();
    S.get_or_init(|| {
        let phi = phi();
        let mut cube = vec![];
        signs([1.0, 1.0, 1.0], &mut cube);
        let mut dodeca = vec![];
        signs([1.0, 1.0, 1.0], &mut dodeca);
        dodeca.extend(cyclic(&[[0.0, 1.0 / phi, phi]]));
        vec![
            Solid { name: "tetrahedron", label: "3,3", v: vec![[1.0, 1.0, 1.0], [1.0, -1.0, -1.0], [-1.0, 1.0, -1.0], [-1.0, -1.0, 1.0]] },
            Solid { name: "cube", label: "4,3", v: cube },
            Solid { name: "octahedron", label: "3,4", v: cyclic(&[[1.0, 0.0, 0.0]]) },
            Solid { name: "dodecahedron", label: "5,3", v: dodeca },
            Solid { name: "icosahedron", label: "3,5", v: cyclic(&[[0.0, 1.0, phi]]) },
        ]
    })
}

pub struct Face { pub n: V3, pub v: Vec<usize> }
/// `f` = (first face, second face if any)
pub struct Edge { pub a: usize, pub b: usize, pub f: (usize, Option<usize>) }
pub struct Hull { pub faces: Vec<Face>, pub edges: Vec<Edge>, pub radius: f64 }

/// Lua 5.4's `x ^ 2` is `x * x` (luai_numpow), not pow()
fn sq(x: f64) -> f64 { x * x }

/// the convex hull of points v around the origin (indices 0-based)
pub fn hull_of(v: &[V3]) -> Hull {
    let n = v.len();
    let mut faces: Vec<Face> = vec![];
    let mut seen: HashSet<(i64, i64, i64)> = HashSet::new();
    let d2 = |a: &V3, b: &V3| sq(a[0] - b[0]) + sq(a[1] - b[1]) + sq(a[2] - b[2]);
    for a in 0..n {
        let mut near = vec![];
        let mut dmin = f64::INFINITY;
        for b in 0..n { if b != a { dmin = dmin.min(d2(&v[a], &v[b])) } }
        for b in 0..n { if b != a && d2(&v[a], &v[b]) < dmin * 1.05 { near.push(b) } }
        for bi in 0..near.len().saturating_sub(1) {
            for ci in bi + 1..near.len() {
                let (aa, bb, cc) = (v[a], v[near[bi]], v[near[ci]]);
                let (ux, uy, uz) = (bb[0] - aa[0], bb[1] - aa[1], bb[2] - aa[2]);
                let (wx, wy, wz) = (cc[0] - aa[0], cc[1] - aa[1], cc[2] - aa[2]);
                let (mut nx, mut ny, mut nz) = (uy * wz - uz * wy, uz * wx - ux * wz, ux * wy - uy * wx);
                let l = (nx * nx + ny * ny + nz * nz).sqrt();
                if l > 1e-9 {
                    nx /= l; ny /= l; nz /= l;
                    let mut d = nx * aa[0] + ny * aa[1] + nz * aa[2];
                    if d < 0.0 { nx = -nx; ny = -ny; nz = -nz; d = -d }
                    let mut ok = d > 1e-6;
                    for m in 0..n { if ok && nx * v[m][0] + ny * v[m][1] + nz * v[m][2] > d + 1e-6 { ok = false } }
                    let key = ((nx * 1000.0 + 0.5).floor() as i64, (ny * 1000.0 + 0.5).floor() as i64, (nz * 1000.0 + 0.5).floor() as i64);
                    if ok && !seen.contains(&key) {
                        seen.insert(key);
                        let mut f: Vec<usize> = vec![];
                        let (mut cx, mut cy, mut cz) = (0.0, 0.0, 0.0);
                        for m in 0..n {
                            if (nx * v[m][0] + ny * v[m][1] + nz * v[m][2] - d).abs() < 1e-6 {
                                f.push(m);
                                cx += v[m][0]; cy += v[m][1]; cz += v[m][2];
                            }
                        }
                        let k = f.len() as f64;
                        cx /= k; cy /= k; cz /= k;
                        let e = v[f[0]];
                        let (e1x, e1y, e1z) = (e[0] - cx, e[1] - cy, e[2] - cz);
                        let (e2x, e2y, e2z) = (ny * e1z - nz * e1y, nz * e1x - nx * e1z, nx * e1y - ny * e1x);
                        let mut ang = vec![0.0; n];
                        for &m in &f {
                            let (px, py, pz) = (v[m][0] - cx, v[m][1] - cy, v[m][2] - cz);
                            ang[m] = (px * e2x + py * e2y + pz * e2z).atan2(px * e1x + py * e1y + pz * e1z);
                        }
                        f.sort_by(|p, q| ang[*p].partial_cmp(&ang[*q]).unwrap_or(std::cmp::Ordering::Equal));
                        faces.push(Face { n: [nx, ny, nz], v: f });
                    }
                }
            }
        }
    }
    let mut edges: Vec<Edge> = vec![];
    let mut idx: HashMap<usize, usize> = HashMap::new();
    for (fi, f) in faces.iter().enumerate() {
        for j in 0..f.v.len() {
            let (a, b) = (f.v[j], f.v[(j + 1) % f.v.len()]);
            let key = a.min(b) * 1000 + a.max(b);
            if let Some(&e) = idx.get(&key) { edges[e].f.1 = Some(fi) }
            else { edges.push(Edge { a, b, f: (fi, None) }); idx.insert(key, edges.len() - 1); }
        }
    }
    let mut radius = 0.0f64;
    for p in v { radius = radius.max((sq(p[0]) + sq(p[1]) + sq(p[2])).sqrt()) }
    Hull { faces, edges, radius }
}

/// the hull of Platonic solid i (0-based), computed once
pub fn hull(i: usize) -> &'static Hull {
    static H: OnceLock<Vec<Hull>> = OnceLock::new();
    &H.get_or_init(|| solids().iter().map(|s| hull_of(&s.v)).collect())[i]
}

/// a solid truncated at its vertices: the points a fraction s along every edge from each end
pub fn truncate(v: &[V3], s: f64) -> Vec<V3> {
    let h = hull_of(v);
    let (mut out, mut seen) = (vec![], HashSet::new());
    let mut add = |p: V3| {
        let key = ((p[0] * 1e4 + 0.5).floor() as i64, (p[1] * 1e4 + 0.5).floor() as i64, (p[2] * 1e4 + 0.5).floor() as i64);
        if seen.insert(key) { out.push(p) }
    };
    for e in &h.edges {
        let (a, b) = (v[e.a], v[e.b]);
        add([a[0] + s * (b[0] - a[0]), a[1] + s * (b[1] - a[1]), a[2] + s * (b[2] - a[2])]);
        add([b[0] + s * (a[0] - b[0]), b[1] + s * (a[1] - b[1]), b[2] + s * (a[2] - b[2])]);
    }
    out
}

/// a convex solid (points v, hull h) in wireframe: k cm per unit, seen from yaw, pitch
pub fn solid(v: &[V3], h: &Hull, k: f64, yaw: f64, pitch: f64) {
    let rot = rotation(yaw, pitch);
    let p: Vec<[f64; 3]> = v.iter().map(|q| { let (x, y, z) = rot(q[0], q[1], q[2]); [x * k, y * k, z] }).collect();
    let (lx, ly, lz) = (-0.45, 0.6, 0.66);
    let mut front = vec![];
    let mut lit = vec![];
    for f in &h.faces {
        let (nx, ny, nz) = rot(f.n[0], f.n[1], f.n[2]);
        front.push(nz > 1e-6);
        let l = nx * lx + ny * ly + nz * lz;
        lit.push(if l > 0.0 { l } else { 0.0 });
    }
    let shown = |e: &Edge| front[e.f.0] || e.f.1.map_or(false, |g| front[g]);
    for e in &h.edges {
        if !shown(e) { ink::moveto(p[e.a][0], p[e.a][1]); ink::lineto(p[e.b][0], p[e.b][1]) }
    }
    ink::stroke(tone(0.35), 0.25, Some(0.75), false, Some("[1.5 1.5] 0"));
    for (fi, f) in h.faces.iter().enumerate() {
        if front[fi] {
            let pts: Vec<Pt> = f.v.iter().map(|&m| [p[m][0], p[m][1]]).collect();
            ink::poly(&pts, true);
            flo(tone(lerp(0.66, 0.93, lit[fi])), 0.75, true);
        }
    }
    for e in &h.edges {
        if shown(e) { ink::moveto(p[e.a][0], p[e.a][1]); ink::lineto(p[e.b][0], p[e.b][1]) }
    }
    st(ink_line(), 0.45);
}

// ---------------------------------------------------------------- the figures
pub struct Figure {
    pub name: &'static str,
    pub w: f64,
    pub max: f64,
    pub r: [f64; 2],
    pub pick: fn(&Rng) -> Vec<f64>,
    pub draw: fn(&Rng, f64, &[f64]),
    pub caption: fn(&[f64]) -> String,
}

fn bit(p: &[f64], i: usize) -> bool { p.get(i).map_or(false, |x| *x != 0.0) }
fn b2f(b: bool) -> f64 { if b { 1.0 } else { 0.0 } }

fn maurer_pick(l: &Rng) -> Vec<f64> { MAURER[l.int(1.0, MAURER.len() as f64) as usize - 1].to_vec() }
fn maurer_draw(rng: &Rng, r: f64, p: &[f64]) { maurer(rng, r, Some([p[0], p[1]])) }
fn maurer_caption(p: &[f64]) -> String {
    let mut s = String::from("$r=\\sin ");
    dd(&mut s, p[0]);
    s.push_str("\\theta$, $d=");
    dd(&mut s, p[1]);
    s.push_str("^\\circ$");
    s
}

const HYPO: [[f64; 3]; 6] = [[5.0, 3.0, 0.8], [7.0, 4.0, 0.9], [8.0, 3.0, 0.75], [5.0, 2.0, 0.9], [7.0, 3.0, 0.7], [9.0, 4.0, 0.85]];
fn hypo_pick(l: &Rng) -> Vec<f64> {
    let p = HYPO[l.int(1.0, HYPO.len() as f64) as usize - 1];
    vec![p[0], p[1], p[2], l.range(0.0, 360.0)]
}
fn hypo_draw(_rng: &Rng, r: f64, p: &[f64]) {
    let (rf, q, d) = (p[0], p[1], p[2] * p[1]);
    let (k, a) = ((rf - q) / q, rf - q);
    let sc = r / rf;
    let laps = q / gcd(rf as i64, q as i64) as f64;
    ink::push();
    ink::scale(sc, None);
    ink::rotate(p[3]);
    ink::circle(0.0, 0.0, rf);
    ink::stroke(tone(0.45), 0.2, Some(0.8), false, Some("[1.2 1.6] 0"));
    let m = (60.0 * laps).floor() as usize;
    let pts: Vec<Pt> = (0..m).map(|i| {
        let t = 2.0 * PI * laps * i as f64 / m as f64;
        [a * t.cos() + d * (k * t).cos(), a * t.sin() - d * (k * t).sin()]
    }).collect();
    sm(&pts, true);
    flo(tone(0.85), 0.5, true);
    sm(&pts, true);
    st1(ink_line(), 0.32, 0.9);
    let t = 0.6f64;
    let (cx, cy) = (a * t.cos(), a * t.sin());
    let (px, py) = (cx + d * (k * t).cos(), cy - d * (k * t).sin());
    ink::circle(cx, cy, q);
    st1(tone(0.35), 0.25, 0.85);
    ink::moveto(cx, cy);
    ink::lineto(px, py);
    st(ink::accent(0.0), 0.3);
    ink::circle(cx, cy, 0.06 * rf / 5.0);
    fl(tone(0.1));
    ink::circle(px, py, 0.09 * rf / 5.0);
    fl(ink::accent(0.0));
    ink::pop();
}
fn hypo_caption(p: &[f64]) -> String {
    let mut s = String::from("hypotrochoid $");
    dd(&mut s, p[0]);
    s.push(':');
    dd(&mut s, p[1]);
    s.push('$');
    s
}

const LISSA: [[f64; 2]; 7] = [[3.0, 2.0], [5.0, 4.0], [3.0, 4.0], [5.0, 6.0], [1.0, 2.0], [5.0, 2.0], [4.0, 3.0]];
fn lissa_pick(l: &Rng) -> Vec<f64> {
    let p = LISSA[l.int(1.0, LISSA.len() as f64) as usize - 1];
    vec![p[0], p[1], b2f(l.chance(0.5))]
}
fn lissa_draw(_rng: &Rng, r: f64, p: &[f64]) {
    let (a, b) = (p[0], p[1]);
    let h = r * 0.7;
    let delta = match p.get(3) { Some(d) => *d, None => if bit(p, 2) { PI / 2.0 / b } else { PI / 4.0 / b } };
    ink::poly(&square(h), true);
    st1(tone(0.5), 0.2, 0.7);
    let m = (60.0 * a.max(b)) as usize;
    let pts: Vec<Pt> = (0..m).map(|i| {
        let t = 2.0 * PI * i as f64 / m as f64;
        [h * (a * t + delta).sin(), h * (b * t).sin()]
    }).collect();
    sm(&pts, true);
    st1(ink_line(), 0.4, 0.9);
}
fn lissa_x(out: &mut String, p: f64) { if p != 1.0 { dd(out, p) } }
fn lissa_caption(p: &[f64]) -> String {
    let mut s = String::from("$x=\\sin ");
    lissa_x(&mut s, p[0]);
    s.push_str("t,\\ y=\\sin ");
    dd(&mut s, p[1]);
    s.push_str("t$");
    s
}

fn polyhedron_pick(l: &Rng) -> Vec<f64> { vec![l.int(1.0, 5.0), l.range(0.0, 360.0), l.range(14.0, 32.0)] }
fn polyhedron_draw(_rng: &Rng, r: f64, p: &[f64]) {
    let i = p[0] as usize - 1;
    let h = hull(i);
    solid(&solids()[i].v, h, r * 0.95 / h.radius, p[1], p[2]);
}
fn polyhedron_caption(p: &[f64]) -> String {
    let s = &solids()[p[0] as usize - 1];
    format!("{} $\\{{{}\\}}$", s.name, s.label)
}

fn torus(p: f64, q: f64) -> impl Fn(f64) -> V3 {
    move |t| { let r = 2.0 + (q * t).cos(); [r * (p * t).cos(), r * (p * t).sin(), (q * t).sin()] }
}
/// the knots (1-based in `p`): name, label, curve
const KNOTS: [(&str, &str); 6] = [("trefoil", "3_1"), ("figure-eight knot", "4_1"), ("cinquefoil", "5_1"),
    ("torus knot", "T_{3,4}"), ("torus knot", "7_1"), ("torus knot", "9_1")];
fn knot_curve(i: usize, t: f64) -> V3 {
    match i {
        0 => torus(2.0, 3.0)(t),
        1 => { let r = 2.0 + (2.0 * t).cos(); [r * (3.0 * t).cos(), r * (3.0 * t).sin(), (4.0 * t).sin()] }
        2 => torus(2.0, 5.0)(t),
        3 => torus(3.0, 4.0)(t),
        4 => torus(2.0, 7.0)(t),
        _ => torus(2.0, 9.0)(t),
    }
}
/// 1-based indices of the segments on top at each crossing
fn crossings(p: &[V3]) -> Vec<usize> {
    let n = p.len();
    let mut out = vec![];
    for i in 1..=n {
        let (a, b) = (p[i - 1], p[i % n]);
        for j in i + 2..=n {
            if !(i == 1 && j == n) {
                let (c, d) = (p[j - 1], p[j % n]);
                let (rx, ry, sx, sy) = (b[0] - a[0], b[1] - a[1], d[0] - c[0], d[1] - c[1]);
                let den = rx * sy - ry * sx;
                if den.abs() > 1e-12 {
                    let t = ((c[0] - a[0]) * sy - (c[1] - a[1]) * sx) / den;
                    let u = ((c[0] - a[0]) * ry - (c[1] - a[1]) * rx) / den;
                    if t >= 0.0 && t < 1.0 && u >= 0.0 && u < 1.0 {
                        let (za, zc) = (lerp(a[2], b[2], t), lerp(c[2], d[2], u));
                        out.push(if za > zc { i } else { j });
                    }
                }
            }
        }
    }
    out
}
fn knot_pick(l: &Rng) -> Vec<f64> { vec![l.int(1.0, KNOTS.len() as f64), l.range(0.0, 360.0)] }
fn knot_draw(_rng: &Rng, r: f64, p: &[f64]) {
    let k = p[0] as usize - 1;
    let n = 240usize;
    let (c, s) = ((p[1] * RAD).cos(), (p[1] * RAD).sin());
    let pp: Vec<V3> = (1..=n).map(|i| {
        let [x, y, z] = knot_curve(k, 2.0 * PI * (i - 1) as f64 / n as f64);
        [(x * c - y * s) * r / 3.05, (x * s + y * c) * r / 3.05, z]
    }).collect();
    let w = 1.5;
    let xy = |q: &V3| -> Pt { [q[0], q[1]] };
    let piece = |i: usize, a: i64, b: i64| -> Vec<Pt> {
        (i as i64 + a..=i as i64 + b).map(|j| xy(&pp[(j - 1).rem_euclid(n as i64) as usize])).collect()
    };
    let tube = |pts: &[Pt], closed: bool| {
        sm(pts, closed);
        st(ink_line(), w);
        sm(pts, closed);
        st(tone(0.55), 0.45);
    };
    let all: Vec<Pt> = pp.iter().map(xy).collect();
    tube(&all, true);
    for i in crossings(&pp) {
        sm(&piece(i, -2, 3), false);
        st(ink::PAPER, w + 2.4);
        tube(&piece(i, -4, 5), false);
    }
}
fn knot_caption(p: &[f64]) -> String {
    let k = KNOTS[p[0] as usize - 1];
    format!("{} ${}$", k.0, k.1)
}

#[derive(Clone, Copy)]
struct Circ { k: f64, x: f64, y: f64 }
fn gasket_gen(circles: &mut Vec<Circ>, kmax: f64, a: Circ, b: Circ, c: Circ, d: Circ) {
    let k = 2.0 * (a.k + b.k + c.k) - d.k;
    if k > kmax { return }
    let x = (2.0 * (a.k * a.x + b.k * b.x + c.k * c.x) - d.k * d.x) / k;
    let y = (2.0 * (a.k * a.y + b.k * b.y + c.k * c.y) - d.k * d.y) / k;
    let e = Circ { k, x, y };
    circles.push(e);
    gasket_gen(circles, kmax, a, b, e, c);
    gasket_gen(circles, kmax, a, c, e, b);
    gasket_gen(circles, kmax, b, c, e, a);
}
fn gasket_pick(l: &Rng) -> Vec<f64> { vec![l.range(0.0, 360.0)] }
fn gasket_draw(_rng: &Rng, r: f64, p: &[f64]) {
    let kmax = 70f64.min(0.9 * r / 0.02);
    let mut circles: Vec<Circ> = vec![];
    let o = Circ { k: -1.0, x: 0.0, y: 0.0 };
    let (a, b, c) = (Circ { k: 2.0, x: -0.5, y: 0.0 }, Circ { k: 2.0, x: 0.5, y: 0.0 }, Circ { k: 3.0, x: 0.0, y: 2.0 / 3.0 });
    circles.push(a); circles.push(b); circles.push(c);
    gasket_gen(&mut circles, kmax, o, a, b, c);
    gasket_gen(&mut circles, kmax, o, a, c, b);
    gasket_gen(&mut circles, kmax, o, b, c, a);
    gasket_gen(&mut circles, kmax, a, b, c, o);
    ink::push();
    ink::scale(r * 0.97, None);
    ink::rotate(p[0]);
    let lims = [2.5, 6.0, 15.0, f64::INFINITY];
    let tones = [0.9, 0.8, 0.68, 0.55];
    for band in 0..4 {
        let mut any = false;
        for c in &circles {
            let lo = if band == 0 { 0.0 } else { lims[band - 1] };
            if c.k > lo && c.k <= lims[band] { ink::circle(c.x, c.y, 1.0 / c.k); any = true }
        }
        if any { fl(tone(tones[band])) }
    }
    for c in &circles { ink::circle(c.x, c.y, 1.0 / c.k) }
    st1(tone(0.12), 0.18, 0.85);
    ink::circle(0.0, 0.0, 1.0);
    st(ink_line(), 0.4);
    ink::pop();
}

fn koch_pick(l: &Rng) -> Vec<f64> { vec![l.range(0.0, 120.0)] }
fn koch_draw(_rng: &Rng, r: f64, p: &[f64]) {
    let tri: Vec<Pt> = (0..3).map(|k| { let a = (90.0 - 120.0 * k as f64 + p[0]) * RAD; [a.cos(), a.sin()] }).collect();
    let mut pts = tri.clone();
    let levels = match p.get(1) { Some(n) => *n, None => if r > 0.9 { 4.0 } else { 3.0 } };
    let mut lv = 1.0;
    while lv <= levels {
        let mut nxt: Vec<Pt> = vec![];
        for i in 0..pts.len() {
            let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
            let (dx, dy) = ((b[0] - a[0]) / 3.0, (b[1] - a[1]) / 3.0);
            let (c60, s60) = (0.5, 3f64.sqrt() / 2.0);
            nxt.push(a);
            nxt.push([a[0] + dx, a[1] + dy]);
            nxt.push([a[0] + dx + dx * c60 - dy * s60, a[1] + dy + dx * s60 + dy * c60]);
            nxt.push([a[0] + 2.0 * dx, a[1] + 2.0 * dy]);
        }
        pts = nxt;
        lv += 1.0;
    }
    ink::push();
    ink::scale(r * 0.97, None);
    ink::poly(&pts, true);
    flo(tone(0.84), 0.8, true);
    ink::poly(&tri, true);
    ink::stroke(ink::accent(0.0), 0.25, Some(0.85), false, Some("[1.5 1.5] 0"));
    ink::poly(&pts, true);
    st(ink_line(), 0.32);
    ink::pop();
}

fn hilbert(order: i64) -> (Vec<(i64, i64)>, i64) {
    let n = 1i64 << order;
    let mut pts = vec![];
    for d in 0..n * n {
        let (mut x, mut y, mut t, mut s) = (0i64, 0i64, d, 1i64);
        while s < n {
            let rx = (t.div_euclid(2)) & 1;
            let ry = (t ^ rx) & 1;
            if ry == 0 {
                if rx == 1 { x = s - 1 - x; y = s - 1 - y }
                std::mem::swap(&mut x, &mut y);
            }
            x += s * rx;
            y += s * ry;
            t = t.div_euclid(4);
            s *= 2;
        }
        pts.push((x, y));
    }
    (pts, n)
}
fn hilbert_pick(l: &Rng) -> Vec<f64> { vec![l.int(0.0, 3.0)] }
fn hilbert_draw(_rng: &Rng, r: f64, p: &[f64]) {
    let order = match p.get(1) { Some(o) => *o as i64, None => if r > 0.85 { 4 } else { 3 } };
    let (pts, n) = hilbert(order);
    let h = r * 0.7;
    let nf = n as f64;
    let out: Vec<Pt> = pts.iter().map(|q| [-h + (q.0 as f64 + 0.5) / nf * 2.0 * h, -h + (q.1 as f64 + 0.5) / nf * 2.0 * h]).collect();
    ink::push();
    ink::rotate(90.0 * p[0]);
    ink::poly(&square(h), true);
    flo(tone(0.9), 0.7, true);
    ink::poly(&square(h), true);
    st1(tone(0.45), 0.2, 0.7);
    ink::poly(&out, false);
    st(ink_line(), 0.4);
    ink::circle(out[0][0], out[0][1], 0.025);
    ink::circle(out[out.len() - 1][0], out[out.len() - 1][1], 0.025);
    fl(ink::accent(0.0));
    ink::pop();
}

fn chords_pick(l: &Rng) -> Vec<f64> { vec![l.int(2.0, 4.0), l.pick(&[100.0, 120.0, 150.0]), l.range(0.0, 360.0)] }
fn chords_draw(_rng: &Rng, rr: f64, p: &[f64]) {
    let (m, n) = (p[0], p[1]);
    let r = rr * 0.95;
    ink::push();
    ink::rotate(p[2]);
    let mut k = 0.0;
    while k <= n - 1.0 {
        let mk = (m * k) % n;
        let (a, b) = (2.0 * PI * k / n, 2.0 * PI * mk / n);
        if k != mk { ink::moveto(r * a.cos(), r * a.sin()); ink::lineto(r * b.cos(), r * b.sin()) }
        k += 1.0;
    }
    st1(tone(0.2), 0.15, 0.6);
    let env: Vec<Pt> = (0..180).map(|i| {
        let t = 2.0 * PI * i as f64 / 180.0;
        [r * (m * t.cos() + (m * t).cos()) / (m + 1.0), r * (m * t.sin() + (m * t).sin()) / (m + 1.0)]
    }).collect();
    sm(&env, true);
    st1(ink::accent(0.0), 0.35, 0.9);
    ink::circle(0.0, 0.0, r);
    st(ink_line(), 0.35);
    ink::pop();
}
fn chords_caption(p: &[f64]) -> String {
    let mut s = String::from("$k\\mapsto ");
    dd(&mut s, p[0]);
    s.push_str("k \\bmod ");
    dd(&mut s, p[1]);
    s.push('$');
    s
}

fn golden_pick(l: &Rng) -> Vec<f64> { vec![l.int(0.0, 3.0), b2f(l.chance(0.5))] }
fn golden_draw(_rng: &Rng, r: f64, p: &[f64]) {
    let phi = phi();
    let (mut x, mut y, mut w, mut h) = (-phi / 2.0, -0.5, phi, 1.0);
    let mut squares: Vec<[f64; 3]> = vec![];
    let mut arc: Vec<Pt> = vec![];
    for i in 0..14 {
        let d = i % 4;
        let (s, cx, cy, a0);
        if d == 0 { s = h; squares.push([x, y, s]); cx = x + s; cy = y; a0 = 180.0; x += s; w -= s }
        else if d == 1 { s = w; squares.push([x, y + h - s, s]); cx = x; cy = y + h - s; a0 = 90.0; h -= s }
        else if d == 2 { s = h; squares.push([x + w - s, y, s]); cx = x + w - s; cy = y + s; a0 = 0.0; w -= s }
        else { s = w; squares.push([x, y, s]); cx = x + s; cy = y + s; a0 = -90.0; y += s; h -= s }
        for j in (if i == 0 { 0 } else { 1 })..=8 {
            let a = (a0 - 90.0 * j as f64 / 8.0) * RAD;
            arc.push([cx + s * a.cos(), cy + s * a.sin()]);
        }
    }
    let k = r * 0.98 / (phi * phi + 1.0).sqrt() * 2.0;
    ink::push();
    ink::rotate(90.0 * p[0]);
    if bit(p, 1) { ink::scale(-1.0, Some(1.0)) }
    ink::scale(k, None);
    let sqr = |q: &[f64; 3]| [[q[0], q[1]], [q[0] + q[2], q[1]], [q[0] + q[2], q[1] + q[2]], [q[0], q[1] + q[2]]];
    ink::poly(&sqr(&squares[0]), true);
    flo(tone(0.88), 0.7, true);
    for q in &squares[..9] { ink::poly(&sqr(q), true) }
    st1(tone(0.4), 0.2, 0.75);
    sm(&arc, false);
    st(ink_line(), 0.45);
    ink::circle(arc[arc.len() - 1][0], arc[arc.len() - 1][1], 0.012 / k * 2.0);
    fl(ink::accent(0.0));
    ink::pop();
}

fn parabola_pick(l: &Rng) -> Vec<f64> { vec![l.range(0.2, 0.3)] }
fn parabola_draw(_rng: &Rng, r: f64, p: &[f64]) {
    let f = p[0];
    let xx = 0.9;
    ink::push();
    ink::scale(r, None);
    ink::translate(0.0, -0.35);
    ink::moveto(-xx, -f);
    ink::lineto(xx, -f);
    ink::stroke(tone(0.35), 0.25, Some(0.8), false, Some("[2 1.5] 0"));
    ink::moveto(0.0, -f - 0.05);
    ink::lineto(0.0, 0.95);
    ink::stroke(tone(0.45), 0.18, Some(0.6), false, Some("[0.6 1.4] 0"));
    for x in [-0.75, -0.5, -0.25, 0.25, 0.5, 0.75] {
        let y = x * x / (4.0 * f);
        if y < 0.95 { ink::moveto(x, 0.95); ink::lineto(x, y); ink::lineto(0.0, f) }
    }
    st1(ink::accent(0.0), 0.22, 0.8);
    let mut pts: Vec<Pt> = vec![];
    for i in 0..=40 {
        let x = -xx + 2.0 * xx * i as f64 / 40.0;
        let y = x * x / (4.0 * f);
        if y <= 1.0 { pts.push([x, y]) }
    }
    sm(&pts, false);
    st(ink_line(), 0.45);
    ink::circle(0.0, f, 0.03);
    fl(ink::accent(-0.1));
    ink::pop();
}

fn petersen_edges() -> Vec<(usize, usize)> {
    let mut e = vec![];
    for i in 0..5 { e.push((i, (i + 1) % 5)); e.push((i, i + 5)); e.push((i + 5, (i + 2) % 5 + 5)); }
    e
}
fn three_colors() -> [u8; 10] {
    let mut adj: Vec<Vec<usize>> = vec![vec![]; 10];
    for (a, b) in petersen_edges() { adj[a].push(b); adj[b].push(a); }
    fn go(v: usize, col: &mut [u8; 10], adj: &[Vec<usize>]) -> bool {
        if v > 9 { return true }
        for c in 1..=3u8 {
            let ok = !adj[v].iter().any(|&u| col[u] == c);
            if ok {
                col[v] = c;
                if go(v + 1, col, adj) { return true }
                col[v] = 0;
            }
        }
        false
    }
    let mut col = [0u8; 10];
    go(0, &mut col, &adj);
    col
}
fn petersen_pick(l: &Rng) -> Vec<f64> { vec![l.range(-8.0, 8.0)] }
fn petersen_draw(_rng: &Rng, r: f64, p: &[f64]) {
    let pcol = three_colors();
    let mut pp = [[0.0; 2]; 10];
    for i in 0..5 {
        let a = (90.0 + 72.0 * i as f64 + p[0]) * RAD;
        pp[i] = [r * 0.92 * a.cos(), r * 0.92 * a.sin()];
        pp[i + 5] = [r * 0.45 * a.cos(), r * 0.45 * a.sin()];
    }
    for (a, b) in petersen_edges() { ink::moveto(pp[a][0], pp[a][1]); ink::lineto(pp[b][0], pp[b][1]) }
    st(ink_line(), 0.4);
    let colors = [tone(0.08), tone(0.5), ink::accent(0.0)];
    for c in 1..=3u8 {
        for v in 0..10 { if pcol[v] == c { ink::circle(pp[v][0], pp[v][1], 0.055) } }
        fl(colors[c as usize - 1]);
    }
    for v in 0..10 { ink::circle(pp[v][0], pp[v][1], 0.055) }
    st(ink_line(), 0.3);
}

fn none_pick(_l: &Rng) -> Vec<f64> { vec![] }
fn vogel_draw(rng: &Rng, r: f64, _p: &[f64]) { phyllotaxis(rng, r / 0.8, &Opts { drift: Some(0.0), ..Opts::default() }) }
fn fay_draw(rng: &Rng, r: f64, _p: &[f64]) { fay(rng, 1.5 * r, None, None) }
fn text(s: &'static str) -> fn(&[f64]) -> String {
    match s {
        "gasket" => |_| "Apollonian gasket".into(),
        "koch" => |_| "Koch snowflake, $\\dim = \\log_3 4$".into(),
        "hilbert" => |_| "Hilbert curve".into(),
        "golden" => |_| "$\\varphi = \\frac{1+\\sqrt5}{2}$".into(),
        "parabola" => |_| "$x^2 = 4fy$".into(),
        "petersen" => |_| "Petersen graph, $\\chi = 3$".into(),
        "vogel" => |_| "$r_n = c\\sqrt n$, $\\theta_n = 137.5^\\circ n$".into(),
        _ => |_| "Fay's butterfly curve".into(),
    }
}

pub fn figures() -> &'static [Figure] {
    static F: OnceLock<Vec<Figure>> = OnceLock::new();
    F.get_or_init(|| vec![
        Figure { name: "polyhedron", w: 3.0, max: 2.0, r: [0.55, 1.15], pick: polyhedron_pick, draw: polyhedron_draw, caption: polyhedron_caption },
        Figure { name: "knot", w: 3.0, max: 2.0, r: [0.55, 1.05], pick: knot_pick, draw: knot_draw, caption: knot_caption },
        Figure { name: "maurer", w: 2.0, max: 2.0, r: [0.7, 1.3], pick: maurer_pick, draw: maurer_draw, caption: maurer_caption },
        Figure { name: "hypotrochoid", w: 2.0, max: 2.0, r: [0.7, 1.3], pick: hypo_pick, draw: hypo_draw, caption: hypo_caption },
        Figure { name: "lissajous", w: 2.0, max: 2.0, r: [0.6, 1.1], pick: lissa_pick, draw: lissa_draw, caption: lissa_caption },
        Figure { name: "gasket", w: 1.0, max: 1.0, r: [0.95, 1.45], pick: gasket_pick, draw: gasket_draw, caption: text("gasket") },
        Figure { name: "koch", w: 1.5, max: 1.0, r: [0.7, 1.25], pick: koch_pick, draw: koch_draw, caption: text("koch") },
        Figure { name: "hilbert", w: 1.5, max: 1.0, r: [0.6, 1.1], pick: hilbert_pick, draw: hilbert_draw, caption: text("hilbert") },
        Figure { name: "chords", w: 1.5, max: 1.0, r: [0.75, 1.3], pick: chords_pick, draw: chords_draw, caption: chords_caption },
        Figure { name: "golden", w: 1.5, max: 1.0, r: [0.75, 1.3], pick: golden_pick, draw: golden_draw, caption: text("golden") },
        Figure { name: "parabola", w: 1.0, max: 1.0, r: [0.7, 1.15], pick: parabola_pick, draw: parabola_draw, caption: text("parabola") },
        Figure { name: "petersen", w: 1.5, max: 1.0, r: [0.6, 1.05], pick: petersen_pick, draw: petersen_draw, caption: text("petersen") },
        Figure { name: "vogel", w: 1.0, max: 1.0, r: [0.6, 1.1], pick: none_pick, draw: vogel_draw, caption: text("vogel") },
        Figure { name: "fay", w: 0.7, max: 1.0, r: [0.7, 1.2], pick: none_pick, draw: fay_draw, caption: text("fay") },
    ])
}

pub fn figure(name: &str) -> Option<&'static Figure> { figures().iter().find(|f| f.name == name) }

/// a fixed order, for the weighted choice
pub const FIGURE_ORDER: [&str; 14] = ["polyhedron", "knot", "maurer", "hypotrochoid", "lissajous", "gasket", "koch", "hilbert",
    "chords", "golden", "parabola", "petersen", "vogel", "fay"];

// ---------------------------------------------------------------- the fleuron
/// for the header: a copper circle with an inscribed triangle, its tangent running out into
/// the head rule; returns the end of the line in page cm
pub fn compass(_rng: &Rng, s: f64, color: Option<Color>) -> (f64, f64) {
    let c = color.unwrap_or_else(|| ink::accent(0.0));
    ink::push();
    ink::scale(s, None);
    let (r, cy) = (0.55, 0.0);
    ink::circle(0.0, cy, r);
    st(c, 0.4);
    let tri: Vec<Pt> = (0..3).map(|k| { let a = (90.0 + 120.0 * k as f64) * RAD; [r * a.cos(), cy + r * a.sin()] }).collect();
    ink::poly(&tri, true);
    st1(c, 0.3, 0.9);
    ink::circle(0.0, cy, 0.07);
    fl(c);
    ink::moveto(0.0, cy + r);
    ink::lineto(2.0, cy + r);
    st(c, 0.4);
    let (rx, ry) = ink::to_page(2.0, cy + r);
    ink::pop();
    (rx, ry)
}

/// rough width of a caption in cm: the letters that show, at 6.5 pt
pub fn caption_width(t: &str) -> f64 {
    let b = t.as_bytes();
    let mut n = 0usize;
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' && i + 1 < b.len() && b[i + 1].is_ascii_alphabetic() {
            while i + 1 < b.len() && b[i + 1].is_ascii_alphabetic() { i += 1 }
            n += 1;
        } else if !b"{}$^_\\".contains(&b[i]) {
            n += 1;
        }
        i += 1;
    }
    n as f64 * 0.115
}

// ---------------------------------------------------------------- the families
pub struct Family {
    pub name: &'static str,
    pub values: Vec<Vec<f64>>,
    pub pick: fn(&Rng) -> Vec<f64>,
    pub title: fn(&[f64]) -> String,
    pub label: fn(&[f64]) -> String,
    pub draw: fn(&Rng, f64, &[f64], &[f64]),
}

fn n_label(v: &[f64]) -> String { let mut s = String::from("$n="); dd(&mut s, v[0]); s.push('$'); s }
fn single(vs: &[f64]) -> Vec<Vec<f64>> { vs.iter().map(|v| vec![*v]).collect() }
fn range360(l: &Rng) -> Vec<f64> { vec![l.range(0.0, 360.0)] }

fn rose_draw(_rng: &Rng, r: f64, v: &[f64], p: &[f64]) {
    let (n, d) = (v[0], v[1]);
    let k = n / d;
    let period = if (n * d) % 2.0 == 1.0 { PI * d } else { 2.0 * PI * d };
    let m = (48.0 * n.max(2.0)) as usize;
    let pts: Vec<Pt> = (0..m).map(|i| {
        let t = period * i as f64 / m as f64;
        let rr = r * 0.95 * (k * t).cos();
        [rr * (t + p[0] * RAD).cos(), rr * (t + p[0] * RAD).sin()]
    }).collect();
    sm(&pts, true);
    flo(tone(0.82), 0.75, true);
    sm(&pts, true);
    st(tone(0.08), 0.4);
    ink::circle(0.0, 0.0, 0.025);
    fl(ink::accent(0.0));
}

fn lissa_fam_pick(l: &Rng) -> Vec<f64> { [[3.0, 2.0], [1.0, 2.0], [5.0, 4.0], [3.0, 4.0]][l.int(1.0, 4.0) as usize - 1].to_vec() }
const DELTA: [&str; 5] = ["0", "\\frac{\\pi}{8}", "\\frac{\\pi}{4}", "\\frac{3\\pi}{8}", "\\frac{\\pi}{2}"];

/// the truncation families' values: [solid, s, kind]; solid 0..3 = cube, octahedron
/// (coordinates 2), dodecahedron, icosahedron; kind 0 = {}, 1 = t{}, 2 = r{}
fn truncation(a: f64, b: f64, sa: f64) -> Vec<Vec<f64>> {
    vec![vec![a, 0.0, 0.0], vec![a, sa, 1.0], vec![a, 0.5, 2.0], vec![b, 1.0 / 3.0, 1.0], vec![b, 0.0, 0.0]]
}
const TRUNC_TEXT: [&str; 4] = ["4,3", "3,4", "5,3", "3,5"];
fn trunc_label(v: &[f64]) -> String {
    format!("${}\\{{{}\\}}$", ["", "t", "r"][v[2] as usize], TRUNC_TEXT[v[0] as usize])
}
fn trunc_base(i: usize) -> Vec<V3> {
    match i {
        0 => solids()[1].v.clone(),
        1 => cyclic(&[[2.0, 0.0, 0.0]]),
        2 => solids()[3].v.clone(),
        _ => solids()[4].v.clone(),
    }
}
pub struct Shape { pub v: Vec<V3>, pub h: Hull }
thread_local! { static SHAPES: RefCell<HashMap<(usize, u64), Rc<Shape>>> = RefCell::new(HashMap::new()); }
fn shape(solid_id: usize, s: f64) -> Rc<Shape> {
    SHAPES.with(|c| {
        if let Some(x) = c.borrow().get(&(solid_id, s.to_bits())) { return x.clone() }
        let base = trunc_base(solid_id);
        let pts = if s > 0.0 { truncate(&base, s) } else { base };
        let h = hull_of(&pts);
        let sh = Rc::new(Shape { v: pts, h });
        c.borrow_mut().insert((solid_id, s.to_bits()), sh.clone());
        sh
    })
}
fn trunc_draw(_rng: &Rng, r: f64, v: &[f64], p: &[f64]) {
    let sh = shape(v[0] as usize, v[1]);
    solid(&sh.v, &sh.h, r * 0.95 / sh.h.radius, p[0], p[1]);
}
fn trunc_pick(l: &Rng) -> Vec<f64> { vec![l.range(0.0, 360.0), l.range(14.0, 30.0)] }

fn hypo_fam_label(v: &[f64]) -> String {
    let t = match v[0] { x if x == 0.25 => "\\frac14", x if x == 0.5 => "\\frac12", x if x == 0.75 => "\\frac34", x if x == 1.0 => "", x if x == 1.25 => "\\frac54", _ => "nil" };
    format!("$d={}r$", t)
}

fn archimedes_draw(_rng: &Rng, rr: f64, v: &[f64], p: &[f64]) {
    let v = v[0];
    let r = rr * 0.95 * (PI / v).cos();
    let (mut inner, mut outer): (Vec<Pt>, Vec<Pt>) = (vec![], vec![]);
    let mut i = 0.0;
    while i <= v - 1.0 {
        let a = p[0] * RAD + 2.0 * PI * i / v;
        inner.push([r * a.cos(), r * a.sin()]);
        outer.push([r / (PI / v).cos() * (a + PI / v).cos(), r / (PI / v).cos() * (a + PI / v).sin()]);
        i += 1.0;
    }
    ink::poly(&outer, true);
    flo(tone(0.88), 0.7, true);
    ink::poly(&inner, true);
    flo(tone(0.74), 0.7, true);
    ink::circle(0.0, 0.0, r);
    st(ink::accent(0.0), 0.3);
    ink::poly(&outer, true);
    st(tone(0.3), 0.25);
    ink::poly(&inner, true);
    st(tone(0.08), 0.4);
}

fn superellipse_draw(_rng: &Rng, r: f64, v: &[f64], p: &[f64]) {
    let v = v[0];
    let e = 2.0 / v;
    let s = r * 0.95 / (if v >= 2.0 { 2f64.powf(0.5 - 1.0 / v) } else { 1.0 });
    let mut pts: Vec<Pt> = vec![];
    for i in 0..96 {
        let t = 2.0 * PI * i as f64 / 96.0;
        let (c, sn) = (t.cos(), t.sin());
        let x = (if c < 0.0 { -1.0 } else { 1.0 }) * c.abs().powf(e);
        let y = (if sn < 0.0 { -1.0 } else { 1.0 }) * sn.abs().powf(e);
        let a = p[0] * RAD;
        pts.push([s * (x * a.cos() - y * a.sin()), s * (x * a.sin() + y * a.cos())]);
    }
    ink::poly(&pts, true);
    flo(tone(0.82), 0.75, true);
    ink::poly(&pts, true);
    st(tone(0.08), 0.4);
}

fn vogel_fam_draw(_rng: &Rng, r: f64, v: &[f64], p: &[f64]) {
    let v = v[0];
    let n = 140usize;
    let golden = v == 137.508;
    for i in 1..=n {
        let a = (p[0] + i as f64 * v) * RAD;
        let rr = r * 0.95 * ((i as f64 - 0.5) / n as f64).sqrt();
        ink::circle(rr * a.cos(), rr * a.sin(), r * 0.035 * (0.6 + 0.6 * (i as f64 / n as f64).sqrt()));
    }
    fl(if golden { tone(0.05) } else { tone(0.2) });
    if golden { ink::circle(0.0, 0.0, r * 0.04); fl(ink::accent(0.0)) }
}

/// a family by name (see `FAMILY_ORDER`); `values` are as described at the top of the file
pub fn family(name: &str) -> Option<Family> {
    let sa_cube = 1.0 / (2.0 + 2f64.sqrt());
    let sa_dodeca = 1.0 / (2.0 + phi());
    Some(match name {
        "rose" => Family { name: "rose",
            values: vec![vec![1.0, 1.0], vec![3.0, 2.0], vec![2.0, 1.0], vec![5.0, 2.0], vec![3.0, 1.0], vec![7.0, 2.0], vec![4.0, 1.0]],
            pick: range360, title: |_| "$r=\\cos k\\theta$".into(),
            label: |v| if v[1] == 1.0 { let mut s = String::from("$k="); dd(&mut s, v[0]); s.push('$'); s }
                else { let mut s = String::from("$k=\\frac{"); dd(&mut s, v[0]); s.push_str("}{"); dd(&mut s, v[1]); s.push_str("}$"); s },
            draw: rose_draw },
        "koch" => Family { name: "koch", values: single(&[0.0, 1.0, 2.0, 3.0, 4.0]),
            pick: |l| vec![l.range(0.0, 120.0)], title: |_| "Koch snowflake".into(), label: n_label,
            draw: |rng, r, v, p| koch_draw(rng, r, &[p[0], v[0]]) },
        "hilbert" => Family { name: "hilbert", values: single(&[1.0, 2.0, 3.0, 4.0]),
            pick: |l| vec![l.int(0.0, 3.0)], title: |_| "Hilbert curves".into(), label: n_label,
            draw: |rng, r, v, p| hilbert_draw(rng, r, &[p[0], v[0]]) },
        "lissajous" => Family { name: "lissajous", values: single(&[0.0, 1.0, 2.0, 3.0, 4.0]),
            pick: lissa_fam_pick,
            title: |p| { let mut s = String::from("$x=\\sin("); lissa_x(&mut s, p[0]); s.push_str("t+\\delta)$, $y=\\sin "); dd(&mut s, p[1]); s.push_str("t$"); s },
            label: |v| format!("$\\delta={}$", DELTA[v[0] as usize]),
            draw: |rng, r, v, p| lissa_draw(rng, r, &[p[0], p[1], 0.0, v[0] * PI / 8.0]) },
        "cube" => Family { name: "cube", values: truncation(0.0, 1.0, sa_cube),
            pick: trunc_pick, title: |_| "from the cube to its dual".into(), label: trunc_label, draw: trunc_draw },
        "dodeca" => Family { name: "dodeca", values: truncation(2.0, 3.0, sa_dodeca),
            pick: trunc_pick, title: |_| "from the dodecahedron to its dual".into(), label: trunc_label, draw: trunc_draw },
        "knots" => Family { name: "knots", values: vec![vec![3.0, 1.0], vec![5.0, 3.0], vec![7.0, 5.0], vec![9.0, 6.0]],
            pick: range360, title: |_| "torus knots".into(),
            label: |v| { let mut s = String::from("$T_{2,"); dd(&mut s, v[0]); s.push_str("}$"); s },
            draw: |rng, r, v, p| knot_draw(rng, r, &[v[1], p[0]]) },
        "chords" => Family { name: "chords", values: single(&[2.0, 3.0, 4.0, 5.0, 6.0]),
            pick: |l| vec![l.pick(&[100.0, 120.0, 150.0]), l.range(0.0, 360.0)],
            title: |p| { let mut s = String::from("$k\\mapsto mk \\bmod "); dd(&mut s, p[0]); s.push('$'); s },
            label: |v| { let mut s = String::from("$m="); dd(&mut s, v[0]); s.push('$'); s },
            draw: |rng, r, v, p| chords_draw(rng, r, &[v[0], p[0], p[1]]) },
        "hypotrochoid" => Family { name: "hypotrochoid", values: single(&[0.25, 0.5, 0.75, 1.0, 1.25]),
            pick: |l| { let r = [[5.0, 3.0], [7.0, 4.0], [5.0, 2.0], [8.0, 3.0]][l.int(1.0, 4.0) as usize - 1]; vec![r[0], r[1], l.range(0.0, 360.0)] },
            title: |p| { let mut s = String::from("hypotrochoids $"); dd(&mut s, p[0]); s.push(':'); dd(&mut s, p[1]); s.push('$'); s },
            label: hypo_fam_label,
            draw: |rng, r, v, p| hypo_draw(rng, r, &[p[0], p[1], v[0], p[2]]) },
        "vogel" => Family { name: "vogel", values: single(&[137.0, 137.3, 137.508, 137.7, 138.0]),
            pick: range360, title: |_| "Vogel's disc: floret $n$ at $n\\alpha$".into(),
            label: |v| if v[0] == 137.508 { "$\\alpha=137.5^\\circ$".into() } else { let mut s = String::from("$"); fmt::fixed(&mut s, v[0], 1); s.push_str("^\\circ$"); s },
            draw: vogel_fam_draw },
        "archimedes" => Family { name: "archimedes", values: single(&[3.0, 4.0, 6.0, 8.0, 12.0, 24.0]),
            pick: range360, title: |_| "Archimedes: $n\\sin\\frac{\\pi}{n}\\to\\pi$".into(), label: n_label, draw: archimedes_draw },
        "superellipse" => Family { name: "superellipse", values: single(&[0.5, 1.0, 1.5, 2.0, 3.0, 6.0]),
            pick: |l| vec![l.range(0.0, 45.0)], title: |_| "$|x|^n+|y|^n=1$".into(),
            label: |v| if v[0] == 0.5 { "$n=\\frac12$".into() } else if v[0] == 1.5 { "$n=\\frac32$".into() } else { n_label(v) },
            draw: superellipse_draw },
        "maurer" => Family { name: "maurer", values: single(&[1.0, 19.0, 29.0, 47.0, 71.0]),
            pick: |l| vec![l.pick(&[2.0, 4.0, 6.0])],
            title: |p| { let mut s = String::from("Maurer roses $r=\\sin "); dd(&mut s, p[0]); s.push_str("\\theta$"); s },
            label: |v| { let mut s = String::from("$d="); dd(&mut s, v[0]); s.push_str("^\\circ$"); s },
            draw: |rng, r, v, p| maurer(rng, r, Some([p[0], v[0]])) },
        _ => return None,
    })
}

pub const FAMILY_ORDER: [&str; 13] = ["cube", "rose", "koch", "knots", "lissajous", "vogel", "chords", "hilbert", "dodeca",
    "archimedes", "superellipse", "hypotrochoid", "maurer"];
