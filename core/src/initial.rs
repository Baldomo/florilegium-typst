// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! initial.lua: a decorated initial, the letter's outline read from an OpenType font
//! (with ttf-parser, standing in for luaotfload's `loadshapes`) and painted like a petal,
//! a vine over and under its strokes, leaves and florets in the gaps, a double frame.
//! Drawn in a local frame of centimeters, [0,S] x [0,S], under the current ink transform.
use crate::flora::{self, Opts};
use crate::ink::{self, tone, Rng, PAPER, WHITE};
use std::collections::HashMap;
use std::f64::consts::PI;

type Pt = [f64; 2];

const RAD: f64 = PI / 180.0;
/// the font file the Lua asks for by default
pub const DEFAULT_FONT: &str = "lmroman10-bold.otf";

fn lerp(a: f64, b: f64, t: f64) -> f64 { a + (b - a) * t }
fn dist(ax: f64, ay: f64, bx: f64, by: f64) -> f64 { let dx = ax - bx; let dy = ay - by; (dx * dx + dy * dy).sqrt() }

// ---------------------------------------------------------------- outlines
/// one segment of luaotfload's outline, in font units: `{x, y, "m"}`, `{x, y, "l"}`,
/// `{x1, y1, x2, y2, x3, y3, "c"}`, `{x1, y1, x2, y2, "q"}`
#[derive(Clone, Debug, PartialEq)]
pub enum Seg {
    M(f64, f64),
    L(f64, f64),
    C(f64, f64, f64, f64, f64, f64),
    Q(f64, f64, f64, f64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    pub boundingbox: [f64; 4],
    pub segments: Vec<Seg>,
}

/// what luaotfload's `loadshapes` returns, by Unicode value (the first glyph of a value wins)
#[derive(Clone, Debug, PartialEq)]
pub struct Shapes {
    pub units: f64,
    pub glyphs: HashMap<u32, Glyph>,
    /// the file name the messages mention
    pub file: String,
}

struct Builder {
    segs: Vec<Seg>,
    bb: Option<[f64; 4]>,
}

impl Builder {
    fn see(&mut self, x: f64, y: f64) {
        match &mut self.bb {
            None => self.bb = Some([x, y, x, y]),
            Some(b) => { b[0] = b[0].min(x); b[1] = b[1].min(y); b[2] = b[2].max(x); b[3] = b[3].max(y) }
        }
    }
}

impl ttf_parser::OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        let (x, y) = (x as f64, y as f64);
        self.see(x, y);
        self.segs.push(Seg::M(x, y));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let (x, y) = (x as f64, y as f64);
        self.see(x, y);
        self.segs.push(Seg::L(x, y));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (x1, y1, x, y) = (x1 as f64, y1 as f64, x as f64, y as f64);
        self.see(x1, y1);
        self.see(x, y);
        self.segs.push(Seg::Q(x1, y1, x, y));
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (x1, y1, x2, y2, x, y) = (x1 as f64, y1 as f64, x2 as f64, y2 as f64, x as f64, y as f64);
        self.see(x1, y1);
        self.see(x2, y2);
        self.see(x, y);
        self.segs.push(Seg::C(x1, y1, x2, y2, x, y));
    }
    fn close(&mut self) {}
}

/// every glyph of the font with a Unicode value, as luaotfload's reader would hand it over
pub fn shapes_from_font(bytes: &[u8]) -> Result<Shapes, String> {
    let face = ttf_parser::Face::parse(bytes, 0).map_err(|e| format!("cannot read the font: {e}"))?;
    let mut byuni: Vec<(u32, ttf_parser::GlyphId)> = vec![];
    if let Some(cmap) = face.tables().cmap {
        for st in cmap.subtables.into_iter().filter(|s| s.is_unicode()) {
            st.codepoints(|cp| if let Some(g) = st.glyph_index(cp) { byuni.push((cp, g)) });
        }
    }
    let mut glyphs = HashMap::new();
    for (cp, id) in byuni {
        if glyphs.contains_key(&cp) { continue }
        let mut b = Builder { segs: vec![], bb: None };
        let rect = face.outline_glyph(id, &mut b);
        let boundingbox = match (&rect, b.bb) {
            (Some(r), _) => [r.x_min as f64, r.y_min as f64, r.x_max as f64, r.y_max as f64],
            _ => [0.0; 4],
        };
        glyphs.insert(cp, Glyph { boundingbox, segments: b.segs });
    }
    Ok(Shapes { units: face.units_per_em() as f64, glyphs, file: DEFAULT_FONT.to_string() })
}

struct Contour {
    ops: Vec<(char, [f64; 6])>,
    poly: Vec<Pt>,
}

/// the contours of one character, as path commands and as flattened polygons,
/// scaled to fit a box of height h and width w (cm) centered at (cx, cy)
fn contours(shapes: &Shapes, ch: char, cx: f64, cy: f64, h: f64, w: f64) -> Result<Vec<Contour>, String> {
    let gl = match shapes.glyphs.get(&(ch as u32)) {
        Some(g) => g,
        None => return Err(format!("no outline for '{}' in {}", ch, shapes.file)),
    };
    let bb = gl.boundingbox;
    let k = (h / (bb[3] - bb[1])).min(w / (bb[2] - bb[0]));
    let (ox, oy) = (cx - k * (bb[0] + bb[2]) / 2.0, cy - k * (bb[1] + bb[3]) / 2.0);
    let p = |x: f64, y: f64| (ox + k * x, oy + k * y);
    let mut out: Vec<Contour> = vec![];
    let (mut lx, mut ly) = (0.0, 0.0);
    for sg in &gl.segments {
        match *sg {
            Seg::M(x, y) => {
                (lx, ly) = p(x, y);
                out.push(Contour { ops: vec![('m', [lx, ly, 0.0, 0.0, 0.0, 0.0])], poly: vec![[lx, ly]] });
            }
            Seg::L(x, y) => if let Some(cur) = out.last_mut() {
                (lx, ly) = p(x, y);
                cur.ops.push(('l', [lx, ly, 0.0, 0.0, 0.0, 0.0]));
                cur.poly.push([lx, ly]);
            },
            Seg::C(..) | Seg::Q(..) => if let Some(cur) = out.last_mut() {
                let (x1, y1, x2, y2, x3, y3);
                match *sg {
                    Seg::C(a, b, c, d, e, f) => {
                        (x1, y1) = p(a, b);
                        (x2, y2) = p(c, d);
                        (x3, y3) = p(e, f);
                    }
                    Seg::Q(a, b, c, d) => {
                        let (qx, qy) = p(a, b);
                        (x3, y3) = p(c, d);
                        (x1, y1) = (lx + 2.0 / 3.0 * (qx - lx), ly + 2.0 / 3.0 * (qy - ly));
                        (x2, y2) = (x3 + 2.0 / 3.0 * (qx - x3), y3 + 2.0 / 3.0 * (qy - y3));
                    }
                    _ => unreachable!(),
                }
                cur.ops.push(('c', [x1, y1, x2, y2, x3, y3]));
                let f3 = ink::bezier3(lx, ly, x1, y1, x2, y2, x3, y3);
                for i in 1..=8 {
                    let (x, y) = f3(i as f64 / 8.0);
                    cur.poly.push([x, y]);
                }
                (lx, ly) = (x3, y3);
            },
        }
    }
    Ok(out)
}

fn trace(cs: &[Contour]) {
    for c in cs {
        for (op, v) in &c.ops {
            match op {
                'm' => ink::moveto(v[0], v[1]),
                'l' => ink::lineto(v[0], v[1]),
                _ => ink::curveto(v[0], v[1], v[2], v[3], v[4], v[5]),
            }
        }
        ink::close();
    }
}

/// even-odd point in the letter
fn inside(cs: &[Contour], x: f64, y: f64) -> bool {
    let mut c = false;
    for ct in cs {
        let p = &ct.poly;
        let n = p.len();
        let mut j = n.wrapping_sub(1);
        for i in 0..n {
            let (xi, yi, xj, yj) = (p[i][0], p[i][1], p[j][0], p[j][1]);
            if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi { c = !c }
            j = i;
        }
    }
    c
}

// ---------------------------------------------------------------- table.sort
/// Lua 5.4's `table.sort` (ltablib.c auxsort, without the randomized pivot, which only
/// matters past 100 elements in a badly unbalanced partition): equal elements end up
/// where Lua puts them.
fn lua_sort<T: Clone>(a: &mut [T], lt: &dyn Fn(&T, &T) -> bool) {
    if a.len() > 1 { auxsort(a, 1, a.len(), lt) }
}

fn auxsort<T: Clone>(a: &mut [T], mut lo: usize, mut up: usize, lt: &dyn Fn(&T, &T) -> bool) {
    while lo < up {
        if lt(&a[up - 1], &a[lo - 1]) { a.swap(lo - 1, up - 1) }
        if up - lo == 1 { break }
        let mut p = (lo + up) / 2;
        if lt(&a[p - 1], &a[lo - 1]) { a.swap(p - 1, lo - 1) }
        else if lt(&a[up - 1], &a[p - 1]) { a.swap(p - 1, up - 1) }
        if up - lo == 2 { break }
        let pivot = a[p - 1].clone();
        a.swap(p - 1, up - 2);
        // partition
        let mut i = lo;
        let mut j = up - 1;
        loop {
            i += 1;
            while lt(&a[i - 1], &pivot) { i += 1 }
            j -= 1;
            while lt(&pivot, &a[j - 1]) { j -= 1 }
            if j < i {
                a.swap(up - 2, i - 1);
                p = i;
                break;
            }
            a.swap(i - 1, j - 1);
        }
        if p - lo < up - p {
            auxsort(a, lo, p - 1, lt);
            lo = p + 1;
        } else {
            auxsort(a, p + 1, up, lt);
            up = p - 1;
        }
    }
}

// ---------------------------------------------------------------- the tile
/// S: side (cm); letter: one character
pub fn draw(rng: &Rng, s: f64, letter: &str, shapes: &Shapes) -> Result<(), String> {
    draw_with(rng, s, letter, shapes, None)
}

/// as `draw`, with the lua's `o.tone`: the tone of the letter's wash
pub fn draw_with(rng: &Rng, s: f64, letter: &str, shapes: &Shapes, o_tone: Option<f64>) -> Result<(), String> {
    let m = 0.07 * s;
    let ch = letter.chars().next().ok_or_else(|| "bad argument #1 to 'codepoint' (out of bounds)".to_string())?;
    let cs = contours(shapes, ch, s * 0.5, s * 0.47, s * 0.66, s * 0.7)?;
    // free space: a grid of the distance to the letter and the frame
    let n = 36;
    let cell = s / n as f64;
    let mut blocked: Vec<Pt> = vec![];
    let mut freecells: Vec<Pt> = vec![];
    for i in 0..n {
        for j in 0..n {
            let (x, y) = ((i as f64 + 0.5) * cell, (j as f64 + 0.5) * cell);
            if inside(&cs, x, y) { blocked.push([x, y]) } else { freecells.push([x, y]) }
        }
    }
    let clearance = |x: f64, y: f64| {
        let mut d = (x - m).min(s - m - x).min(y - m).min(s - m - y);
        for b in &blocked {
            let e = dist(x, y, b[0], b[1]) - cell * 0.7;
            if e < d { d = e }
        }
        d
    };

    // the ground: a pale glaze and a double frame
    ink::layer(1);
    ink::poly(&[[0.0, 0.0], [s, 0.0], [s, s], [0.0, s]], true);
    ink::clip();
    ink::flood(tone(0.975), 0.0, 0.0, s, s);
    flora::mottle(rng, 0.0, 0.0, s, s, 6, s * 0.2, 0.93, 1.0, Some(0));
    ink::unclip();
    ink::poly(&[[0.0, 0.0], [s, 0.0], [s, s], [0.0, s]], true);
    ink::stroke(tone(0.1), 0.8, None, false, None);
    ink::poly(&[[m * 0.5, m * 0.5], [s - m * 0.5, m * 0.5], [s - m * 0.5, s - m * 0.5], [m * 0.5, s - m * 0.5]], true);
    ink::stroke(tone(0.15), 0.3, Some(0.8), false, None);

    // the vine: from the lower-left corner up across the letter to the upper right
    let f = ink::fbm1(rng, 2);
    let ctrl: Vec<Pt> = vec![
        [m * 1.2, m * 1.2],
        { let x = s * rng.range(0.14, 0.22); let y = s * rng.range(0.3, 0.4); [x, y] },
        { let x = s * rng.range(0.32, 0.42); let y = s * rng.range(0.5, 0.58); [x, y] },
        { let x = s * rng.range(0.58, 0.68); let y = s * rng.range(0.42, 0.52); [x, y] },
        { let x = s * rng.range(0.74, 0.82); let y = s * rng.range(0.62, 0.72); [x, y] },
        [s - m * 1.6, s - m * 1.4],
    ];
    let mut pts = ink::catmull(&ctrl, 14);
    let np = pts.len();
    for (i0, p) in pts.iter_mut().enumerate() {
        let u = i0 as f64 / (np - 1) as f64;
        p[1] += 0.03 * s * f(u * 6.0);
    }
    let (w0, w1) = (0.026 * s, 0.01 * s);
    // 1-based, as the Lua
    let width = |i: usize| lerp(w0, w1, (i - 1) as f64 / (np - 1) as f64);
    ink::layer(2);
    flora::stem(&pts, w0, w1, None, None, None);
    // where it crosses the letter, it goes over and under in turn
    let mut runs: Vec<[usize; 2]> = vec![];
    let mut in_run = false;
    for (i0, p) in pts.iter().enumerate() {
        let i = i0 + 1;
        if inside(&cs, p[0], p[1]) {
            if !in_run { runs.push([i, i]); in_run = true } else { runs.last_mut().unwrap()[1] = i }
        } else { in_run = false }
    }
    // the strands that pass over: drawn again on top of the letter, inside its
    // outline only, with a thin halo of paper that parts them from the letter
    let mut over = rng.chance(0.5);
    for r in &runs {
        if over {
            let (a, b) = (1.max(r[0] as i64 - 2) as usize, np.min(r[1] + 2));
            let seg: Vec<Pt> = (a..=b).map(|i| pts[i - 1]).collect();
            ink::layer(4);
            trace(&cs);
            ink::clip();
            ink::smooth(&seg, false, None);
            ink::stroke(PAPER, (width(a) + width(b)) / 2.0 * ink::unit() * ink::CM + 1.0, None, false, None);
            flora::stem(&seg, width(a), width(b), None, None, None);
            ink::unclip();
        }
        over = !over;
    }
    // a bud at its tip
    {
        let (e, q) = (pts[np - 1], pts[np - 4]);
        ink::layer(5);
        ink::push();
        ink::translate(e[0], e[1]);
        ink::rotate((e[1] - q[1]).atan2(e[0] - q[0]) / RAD - 90.0);
        flora::bud(rng, 0.13 * s, &Opts::default());
        ink::pop();
    }

    // the letter: wash, granulation, pooled edges, light lifted out
    let t0 = match o_tone { Some(t) => t, None => rng.range(0.16, 0.22) };
    ink::layer(3);
    trace(&cs);
    ink::fill(tone(t0), None, false);
    trace(&cs);
    ink::clip();
    flora::mottle(rng, 0.0, 0.0, s, s, 22, s * 0.06, t0 - 0.12, t0 + 0.2, Some(20));
    for k in 1..=9 {
        let x = s * (0.15 + 0.7 * (k as f64 - 0.5) / 9.0) + rng.sym(0.02 * s);
        let y0 = s * rng.range(0.1, 0.25);
        let y1 = s * rng.range(0.72, 0.9);
        flora::streak3(x, y0, x + 0.02 * s, lerp(y0, y1, 0.35), x - 0.02 * s, lerp(y0, y1, 0.7), x + rng.sym(0.03 * s), y1,
            0.018 * s, 0.003 * s, WHITE, Some(rng.range(0.25, 0.45)), false);
    }
    trace(&cs);
    ink::stroke(tone(0.04), 0.07 * s * ink::unit() * ink::CM, Some(0.45), true, None);
    ink::unclip();
    trace(&cs);
    ink::stroke(tone(0.04), 0.35, Some(0.7), false, None);

    // leaves along the vine where they fit, alternating sides
    ink::layer(4);
    let mut sacc = 0.0;
    let mut acc = s * rng.range(0.08, 0.14);
    let mut side = 1.0;
    for i in 2..=np - 3 {
        let (pi, pm, pp) = (pts[i - 1], pts[i - 2], pts[i]);
        sacc += dist(pi[0], pi[1], pm[0], pm[1]);
        if sacc >= acc {
            acc += s * rng.range(0.11, 0.16);
            side = -side;
            let (tx, ty) = (pp[0] - pm[0], pp[1] - pm[1]);
            let a = ty.atan2(tx) / RAD + side * rng.range(40.0, 60.0);
            let l = s * rng.range(0.16, 0.22);
            let mut ok = true;
            for u in [0.35, 0.65, 0.95] {
                let x = pi[0] + l * u * (a * RAD).cos();
                let y = pi[1] + l * u * (a * RAD).sin();
                if inside(&cs, x, y) || clearance(x, y) < 0.03 * s { ok = false }
            }
            if ok {
                ink::push();
                ink::translate(pi[0], pi[1]);
                ink::rotate(a - 90.0);
                ink::scale(l, None);
                flora::leaflet(rng, &Opts { bend: Some(-side * 0.2), ..Opts::default() });
                ink::pop();
            }
        }
    }

    // flowers in the largest gaps: a blossom, and florets on hairlines from the vine
    ink::layer(5);
    let mut spots: Vec<[f64; 3]> = vec![];
    for c in &freecells {
        let cl = clearance(c[0], c[1]);
        if cl > 0.06 * s { spots.push([c[0], c[1], cl]) }
    }
    lua_sort(&mut spots, &|a, b| a[2] > b[2]);
    let mut placed: Vec<[f64; 3]> = vec![];
    let mut nb = 0;
    for sp in &spots {
        if nb >= 4 { break }
        let mut far = true;
        for p in &placed {
            if dist(sp[0], sp[1], p[0], p[1]) < p[2] + sp[2] + 0.04 * s { far = false }
        }
        if far {
            nb += 1;
            let r = (sp[2] * 0.85).min(0.12 * s);
            placed.push([sp[0], sp[1], r]);
            // a hairline stalk back to the vine, if the vine is near
            let mut best = f64::INFINITY;
            let mut bp: Option<Pt> = None;
            for p in &pts {
                let d = dist(p[0], p[1], sp[0], sp[1]);
                if d < best { best = d; bp = Some(*p) }
            }
            if let Some(bp) = bp {
                if best < 0.3 * s {
                    ink::layer(2);
                    ink::moveto(bp[0], bp[1]);
                    ink::curveto(lerp(bp[0], sp[0], 0.3), lerp(bp[1], sp[1], 0.1) + 0.03 * s, lerp(bp[0], sp[0], 0.7), sp[1] - 0.02 * s, sp[0], sp[1]);
                    ink::stroke(tone(0.12), 0.35, None, false, None);
                    ink::layer(5);
                }
            }
            ink::push();
            ink::translate(sp[0], sp[1]);
            if r > 0.07 * s && nb <= 2 {
                flora::blossom(rng, r, &Opts::default());
            } else {
                let star = rng.chance(0.5);
                flora::floret(rng, r * 0.8, &Opts { star: Some(star), ..Opts::default() });
            }
            ink::pop();
        }
    }
    // a copper sparkle in a corner
    ink::layer(6);
    ink::push();
    ink::translate(s - m * 2.2, m * 2.2);
    flora::sparkle(rng, 0.045 * s, &Opts::default());
    ink::pop();
    Ok(())
}
