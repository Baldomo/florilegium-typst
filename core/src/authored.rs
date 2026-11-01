// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! authored.lua: the authored composition of florilegium.
//!
//! Everything here is placed by hand, for A4 with a 68 mm left margin (the margin column
//! is x in [0.3, 6.4] cm, the page 29.7 cm tall): three stems through hand-picked control
//! points, leaves at chosen fractions of their length, and motifs at chosen points, each
//! joined to a chosen stem by a branch. Only the parameters are sampled (every petal,
//! lobe and vein, and a jitter of a millimeter or two on the control points).
use crate::flora::{self, Opts};
use crate::grow::{self, G, PageResult};
use crate::ink::{self, Rng};
use std::f64::consts::PI;

type Pt = [f64; 2];

const RAD: f64 = PI / 180.0;

fn norm(x: f64, y: f64) -> (f64, f64) { let l = (x * x + y * y).sqrt(); if l < 1e-9 { (0.0, 1.0) } else { (x / l, y / l) } }
fn deg(x: f64, y: f64) -> f64 { y.atan2(x) / RAD }

// ---------------------------------------------------------------- the score
const ROOT: [f64; 2] = [1.35, 0.75];

struct StemDef { pts: &'static [[f64; 2]], w: [f64; 2], tip: (&'static str, f64, f64) }

const STEMS: [(&str, StemDef); 3] = [
    ("A", StemDef { pts: &[[1.35, 0.75], [2.3, 3.2], [1.7, 6.6], [2.9, 10.2], [2.1, 13.8], [3.2, 17.6], [2.3, 21.2], [2.9, 24.4], [2.7, 25.6]],
        w: [0.085, 0.03], tip: ("flower", 1.7, 90.0) }),
    ("B", StemDef { pts: &[[1.55, 0.75], [3.1, 4.2], [3.7, 8.3], [2.7, 12.2], [3.9, 15.8], [4.4, 18.4]],
        w: [0.065, 0.028], tip: ("lily", 1.15, 70.0) }),
    ("C", StemDef { pts: &[[1.15, 0.75], [0.95, 3.6], [2.1, 7.3], [1.3, 10.6], [1.4, 12.0]],
        w: [0.06, 0.028], tip: ("flower", 0.95, 90.0) }),
];

// stem index, fraction of its length, side (+1 left), length, lobed
const LEAVES: [(usize, f64, f64, f64, bool); 14] = [
    (0, 0.08, -1.0, 2.3, true), (0, 0.20, 1.0, 2.2, false), (0, 0.33, -1.0, 2.0, true), (0, 0.47, 1.0, 1.9, true),
    (0, 0.60, -1.0, 1.8, false), (0, 0.72, 1.0, 1.6, true), (0, 0.84, -1.0, 1.4, false),
    (1, 0.15, 1.0, 2.1, true), (1, 0.33, -1.0, 1.9, false), (1, 0.52, 1.0, 1.8, true), (1, 0.72, -1.0, 1.5, true),
    (2, 0.22, 1.0, 1.8, false), (2, 0.46, -1.0, 1.7, true), (2, 0.70, 1.0, 1.4, false),
];

// stem, fraction where the branch leaves, target center (cm), motif, size
const BRANCHES: [(usize, f64, f64, f64, &str, f64); 28] = [
    (0, 0.90, 0.95, 27.9, "sprig", 1.6),
    (0, 0.95, 4.75, 24.9, "bud", 0.7),
    (0, 0.86, 4.85, 23.1, "flower", 1.0),
    (0, 0.78, 0.95, 22.4, "umbel", 1.3),
    (0, 0.70, 0.9, 19.3, "blossoms", 0.75),
    (0, 0.74, 5.35, 21.4, "blossoms", 0.65),
    (1, 0.86, 5.55, 16.6, "cyme", 1.9),
    (0, 0.55, 0.8, 16.0, "bud", 0.7),
    (2, 0.95, 0.75, 14.6, "cyme", 1.5),
    (0, 0.60, 5.25, 13.4, "blossoms", 0.8),
    (2, 0.55, 0.65, 11.4, "bud", 0.6),
    (1, 0.45, 5.35, 10.6, "umbel", 1.4),
    (2, 0.62, 0.75, 9.0, "bells", 0.9),
    (1, 0.28, 5.2, 6.6, "flower", 1.05),
    (2, 0.30, 0.7, 5.6, "sprig", 2.0),
    (0, 0.22, 4.7, 3.2, "cyme", 1.6),
    (1, 0.62, 5.6, 12.0, "tendril", 0.4),
    (0, 0.40, 1.0, 13.2, "tendril", 0.35),
    (0, 0.97, 3.9, 28.3, "cyme", 1.2),
    (0, 0.93, 5.25, 25.4, "blossoms", 0.6),
    (0, 0.82, 1.0, 24.7, "leaf", 1.4),
    (1, 0.80, 5.7, 19.9, "bud", 0.6),
    (1, 0.64, 5.05, 14.9, "leaf", 1.4),
    (1, 0.38, 5.35, 8.8, "cyme", 1.4),
    (2, 0.50, 0.75, 7.4, "blossoms", 0.6),
    (0, 0.10, 0.85, 3.1, "leaf", 1.5),
    (1, 0.16, 5.6, 4.4, "bud", 0.6),
    (2, 0.85, 2.6, 11.5, "bud", 0.55),
];

const GHOSTS: [[f64; 3]; 2] = [[3.4, 5.4, 1.5], [2.7, 16.9, 1.25]];
// kind, x, y (cm), size, heading (degrees); the snail: stem, fraction, side, size
const VISITORS: [(&str, f64, f64, f64, f64); 2] = [("butterfly", 4.3, 20.6, 1.3, 100.0), ("bee", 1.6, 26.6, 0.75, 30.0)];
const SNAIL: (usize, f64, f64, f64) = (2, 0.35, 1.0, 0.8);
const ACCENTS: [(&str, f64, f64); 5] = [("rosette", 0.45, 4.2), ("sparkle", 0.5, 9.9), ("sparkle", 0.45, 18.3), ("rosette", 0.5, 24.3), ("sparkle", 5.9, 1.6)];

// the footer: the stem runs from the root to this fraction of the text width;
// motifs along it: x (cm), motif, size
const FOOT_END: f64 = 0.45;
const FOOT: [(f64, &str, f64); 4] = [(3.6, "cyme", 1.1), (5.3, "blossoms", 0.5), (7.6, "cyme", 1.0), (9.6, "blossoms", 0.42)];

// ---------------------------------------------------------------- drawing
/// points along a polyline at a fraction of its length, with the unit tangent
fn at(pts: &[Pt], f: f64) -> (f64, f64, f64, f64) {
    let (s, tot) = ink::arclen(pts);
    let want = f * tot;
    for i in 1..pts.len() {
        if s[i] >= want {
            let (tx, ty) = norm(pts[i][0] - pts[i - 1][0], pts[i][1] - pts[i - 1][1]);
            return (pts[i][0], pts[i][1], tx, ty);
        }
    }
    let n = pts.len();
    let (tx, ty) = norm(pts[n - 1][0] - pts[n - 2][0], pts[n - 1][1] - pts[n - 2][1]);
    (pts[n - 1][0], pts[n - 1][1], tx, ty)
}

fn place(layer: i32, x: f64, y: f64, a: f64, f: impl FnOnce()) {
    ink::layer(layer);
    ink::push();
    ink::translate(x, y);
    ink::rotate(a - 90.0);
    f();
    ink::pop();
}

pub fn page(g: &G) -> Result<PageResult, String> {
    let rng = Rng::new(g.seed * 7919.0 + 17.0);
    let jit = rng.fork();
    grow::theme(g.theme.as_deref())?;
    let th = grow::current();
    if let Some(own) = th.page { return own(g); }
    let fl = &th.flora;
    let ly = grow::layers();
    let motif = |name: &str| grow::motif(name).ok_or_else(|| format!("florilegium: no motif '{name}'"));
    ink::set_transform(ink::page_transform(if g.mirror { Some(g.pw) } else { None }));

    // stems
    let mut stems: Vec<Vec<Pt>> = vec![];
    for (_, st) in STEMS.iter() {
        let mut cp: Vec<Pt> = vec![];
        for (i, p) in st.pts.iter().enumerate() {
            let x = p[0] + if i > 0 { jit.sym(0.12) } else { 0.0 };
            let y = p[1] + if i > 0 { jit.sym(0.12) } else { 0.0 };
            cp.push([x, y]);
        }
        stems.push(ink::catmull(&cp, 10));
    }
    ink::layer(ly.stem);
    for k in [2usize, 1, 0] {
        fl.stem(&stems[k], STEMS[k].1.w[0], STEMS[k].1.w[1], None, None, None);
    }
    // the scroll at the root
    {
        let mut pts: Vec<Pt> = vec![];
        for i in 0..=40 {
            let t = i as f64 / 40.0;
            let a = PI * 0.5 + t * 1.4 * 2.0 * PI;
            let r = 0.55 * (1.0 - 0.85 * t).powf(1.3);
            pts.push([ROOT[0] - 0.55 - r * a.cos(), ROOT[1] + 0.05 + r * a.sin()]);
        }
        fl.stem(&pts, 0.06, 0.012, None, None, None);
    }
    // footer stem
    let fx1 = g.textleft + (g.textright - g.textleft) * FOOT_END;
    let mut foot: Vec<Pt> = vec![];
    {
        let f = ink::fbm1(&jit, 2);
        let mut x = ROOT[0];
        while x <= fx1 {
            let mut u = 1f64.min((x - ROOT[0]) / 1.5);
            u = u * u * (3.0 - 2.0 * u);
            foot.push([x, ROOT[1] + 0.1 + (0.95 - ROOT[1] - 0.1 + 0.2 * f(x / 3.5)) * u]);
            x += 0.2;
        }
        fl.stem(&foot, 0.065, 0.01, None, None, None);
    }

    // terminal flowers
    for (k, (_, st)) in STEMS.iter().enumerate() {
        let pts = &stems[k];
        let (p, q) = (pts[pts.len() - 1], pts[pts.len() - 4]);
        let (ax, ay) = norm(p[0] - q[0], p[1] - q[1]);
        let (kind, s, a) = st.tip;
        let m = motif(kind)?;
        let r = rng.fork();
        place(m.layer, p[0] + ax * 0.2, p[1] + ay * 0.2, a, || (m.draw)(fl, &r, s, None));
    }

    // ghosts behind everything
    for gh in GHOSTS.iter() {
        let r = rng.fork();
        place(ly.ghost, gh[0], gh[1], 90.0, || flora::ghost(Some(0.62), || fl.sized("flower", &r, gh[2], &Opts::default())));
    }

    // leaves at the nodes
    for lf in LEAVES.iter() {
        let (x, y, tx, ty) = at(&stems[lf.0], lf.1);
        let a = deg(tx, ty) + lf.2 * jit.range(40.0, 55.0);
        let l = lf.3 * 1.12 * jit.range(0.92, 1.05);
        let r = rng.fork();
        let bend = lf.2 * jit.range(0.08, 0.3);
        place(ly.leaf, x, y, a, || {
            ink::scale(l, None);
            let depth = if lf.4 { jit.range(0.45, 0.75) } else { 0.0 };
            fl.scaled("leaf", &r, &Opts { w: Some(0.26), depth: Some(depth), bend: Some(bend), ..Opts::default() });
        });
    }
    // leaflets along the footer, smaller toward its end
    {
        let (s, tot) = ink::arclen(&foot);
        let mut side = 1.0;
        let mut nexts = 0.7;
        let n = foot.len();
        for i in 2..=(n as i64 - 2) {
            let i = i as usize;
            if s[i - 1] >= nexts {
                let u = s[i - 1] / tot;
                nexts = nexts + (0.55 + 0.8 * u) * jit.range(0.85, 1.15);
                side = -side;
                let (tx, ty) = norm(foot[i][0] - foot[i - 2][0], foot[i][1] - foot[i - 2][1]);
                let l = (0.9 - 0.45 * u) * jit.range(0.85, 1.1);
                let a = deg(tx, ty) + side * jit.range(35.0, 55.0);
                let r = rng.fork();
                let bend = -side * 0.2;
                place(ly.leaf, foot[i - 1][0], foot[i - 1][1], a, || {
                    ink::scale(l, None);
                    fl.scaled("leaflet", &r, &Opts { bend: Some(bend), ..Opts::default() });
                });
            }
        }
    }

    // branches and their motifs
    let grow_branch = |sx: f64, sy: f64, tx: f64, ty: f64, px: f64, py: f64, kind: &str, s: f64, w0: f64| -> Result<(), String> {
        let m = motif(kind)?;
        let (dx, dy) = norm(px - sx, py - sy);
        let (ax, ay) = norm(dx, dy + m.upright.unwrap_or(0.0));
        let (mut ox, mut oy) = (px - ax * (m.off)(s), py - ay * (m.off)(s));
        if m.hang { ox = px - dx * 0.2 * s; oy = py + 0.45 * s; }
        let bpts = flora::branch_path(&jit, sx, sy, tx, ty, ox, oy, if m.hang { dx } else { ax }, if m.hang { -0.3 } else { ay });
        ink::layer(ly.stem);
        fl.stem(&bpts, w0, 0.014, None, None, None);
        let d = ((ox - sx) * (ox - sx) + (oy - sy) * (oy - sy)).sqrt();
        if d > 1.0 {
            let i = (bpts.len() as f64 * 0.4).floor() as usize;
            let (p, q) = (bpts[i - 1], bpts[i]);
            let sgn = if jit.chance(0.5) { 1.0 } else { -1.0 };
            let a = deg(q[0] - p[0], q[1] - p[1]) + sgn * jit.range(40.0, 60.0);
            let ls = jit.range(0.45, 0.7);
            let r = rng.fork();
            place(ly.leaf, p[0], p[1], a, || {
                ink::scale(ls, None);
                fl.scaled("leaflet", &r, &Opts::default());
            });
        }
        let r = rng.fork();
        place(m.layer, ox, oy, if m.hang { 90.0 } else { deg(ax, ay) }, || (m.draw)(fl, &r, s, None));
        Ok(())
    };
    for b in BRANCHES.iter() {
        let (x, y, tx, ty) = at(&stems[b.0], b.1);
        let px = b.2 + jit.sym(0.1);
        let py = b.3 + jit.sym(0.1);
        grow_branch(x, y, tx, ty, px, py, b.4, b.5, 0.035)?;
    }
    for b in FOOT.iter() {
        if b.0 < fx1 - 0.5 {
            let (mut best, mut bi) = (f64::INFINITY, 0usize);
            for (i, p) in foot.iter().enumerate() {
                let d = (p[0] - (b.0 - 0.4)).abs();
                if d < best { best = d; bi = i; }
            }
            let (p, q) = (foot[bi], foot[(foot.len() - 1).min(bi + 1)]);
            let (tx, ty) = norm(q[0] - p[0], q[1] - p[1]);
            grow_branch(p[0], p[1], tx, ty, b.0, p[1] + 0.45, b.1, b.2, 0.025)?;
        }
    }

    // accents and the footer's copper tail
    for ac in ACCENTS.iter() {
        let r = rng.fork();
        let size = if ac.0 == "sparkle" { 0.13 } else { 0.12 };
        place(ly.accent, ac.1, ac.2, 90.0, || fl.sized(ac.0, &r, size, &Opts::default()));
    }
    {
        let e = foot[foot.len() - 1];
        let x2 = e[0] + 1.9;
        ink::layer(ly.accent);
        ink::moveto(e[0], e[1]);
        ink::curveto(e[0] + 0.6, e[1] + 0.05, x2 - 0.6, e[1] - 0.05, x2, e[1]);
        ink::stroke(ink::accent(0.0), 0.3, Some(0.8), false, None);
        place(ly.accent, x2 + 0.18, e[1], 90.0, || fl.sized("sparkle", &rng.fork(), 0.12, &Opts { rays: Some(false), ..Opts::default() }));
        ink::circle(x2 + 0.45, e[1], 0.025);
        ink::fill(ink::accent(0.0), None, false);
    }

    // visitors last, from their own stream, so the rest of the page does not change
    if g.visitors != Some(false) {
        let vr = Rng::new(g.seed * 7919.0 + 103.0);
        for v in VISITORS.iter() {
            let r = vr.fork();
            place(ly.accent, v.1, v.2, v.4, || fl.sized(v.0, &r, v.3, &Opts::default()));
        }
        let (x, y, tx, ty) = at(&stems[SNAIL.0], SNAIL.1);
        let r = vr.fork();
        place(ly.accent, x + ty * SNAIL.2 * 0.03, y - tx * SNAIL.2 * 0.03, deg(tx, ty), || {
            ink::scale(SNAIL.2, Some(1.0));
            fl.sized("snail", &r, SNAIL.3, &Opts::default());
        });
    }

    // a safety clip: the column (a little past its edge) and the footer band
    ink::layer(0);
    ink::poly(&[[0.0, 0.0], [g.colright + 0.25, 0.0], [g.colright + 0.25, g.ph], [0.0, g.ph]], true);
    let top = 2.0f64.min(g.footy - 0.2);
    ink::poly(&[[g.colright, 0.0], [fx1 + 3.0, 0.0], [fx1 + 3.0, top], [g.colright, top]], true);
    ink::clip();
    ink::layer(99);
    ink::unclip();
    ink::flush();
    Ok(PageResult::default())
}
