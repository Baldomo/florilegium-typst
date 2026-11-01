// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! theme-math.lua: a theme that is not a plant: mathematics cascading down the margin and
//! across the bottom of the page. Shapes, symbols and equations (set as TeX captions),
//! overlapping a little so they blend: one large figure where the column meets the bottom,
//! families flowing down the column toward it, equations along the bottom and under the text,
//! and everywhere else, largest hole first, figures, symbols and dots.
use crate::figures::{self, FAMILY_ORDER, FIGURE_ORDER};
use crate::flora::Opts;
use crate::grow::{self, Caption, Flora, GPrim, MarkKind, PageResult, ThemeDef, G};
use crate::ink::{self, Event, EventData, Rng, Tag};
use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::rc::Rc;

fn tone(t: f64) -> ink::Color { ink::tone(t) }

const GLYPHS: [&str; 53] = [
    "\\mathrm{A}", "\\mathrm{M}", "\\mathrm{Q}", "\\mathrm{B}", "\\mathrm{D}", "\\mathcal{F}", "\\mathcal{L}", "\\mathcal{R}",
    "x", "y", "n", "k", "e", "i", "z",
    "0", "1", "2", "3", "4", "5", "7", "8", "9",
    "+", "=", "-", "\\times", "\\pm", "(", ")", "[", "]", "!", "\\approx", "\\in", "\\le",
    "\\pi", "\\varphi", "\\lambda", "\\theta", "\\Omega", "\\Delta", "\\Sigma", "\\partial", "\\nabla", "\\infty", "\\aleph",
    "\\displaystyle\\int", "\\displaystyle\\sum", "\\displaystyle\\prod", "\\displaystyle\\oint", "\\sqrt{\\;}",
];

const EQUATIONS: [&str; 23] = [
    "e^{i\\pi}+1=0", "a^2+b^2=c^2", "\\varphi^2=\\varphi+1", "V-E+F=2", "i^2=-1",
    "\\sum_{n\\ge1}\\frac{1}{n^2}=\\frac{\\pi^2}{6}", "\\int_{-\\infty}^{\\infty}e^{-x^2}dx=\\sqrt{\\pi}",
    "F_{n+1}=F_n+F_{n-1}", "\\cos^2\\theta+\\sin^2\\theta=1", "x=\\frac{-b\\pm\\sqrt{b^2-4ac}}{2a}",
    "e^x=\\sum_{n\\ge0}\\frac{x^n}{n!}", "e=\\sum_{n\\ge0}\\frac{1}{n!}", "\\frac{d}{dx}e^x=e^x",
    "1+2+\\cdots+n=\\frac{n(n+1)}{2}", "\\zeta(s)=\\prod_p\\frac{1}{1-p^{-s}}", "z\\mapsto z^2+c",
    "\\lim_{n\\to\\infty}\\bigl(1+\\frac1n\\bigr)^n=e", "\\pi\\approx\\frac{355}{113}", "\\dim=\\log_3 4",
    "r=\\cos k\\theta", "\\oint_C f\\,dz=0", "2^{\\aleph_0}>\\aleph_0", "n\\sin\\frac{\\pi}{n}\\to\\pi",
];

/// the width of TeX at 10 pt, in cm, roughly: the characters that show
fn width(t: &str) -> f64 {
    let b = t.as_bytes();
    let mut n = 0usize;
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' && i + 1 < b.len() && b[i + 1].is_ascii_alphabetic() {
            while i + 1 < b.len() && b[i + 1].is_ascii_alphabetic() { i += 1 }
            n += 1;
        } else if !matches!(b[i], b'{' | b'}' | b'$' | b'^' | b'_' | b'\\') {
            n += 1;
        }
        i += 1;
    }
    n as f64 * 0.2
}

fn new_event(events: &mut Vec<Event>) -> Event {
    let e = Rc::new(RefCell::new(EventData { t0: 0.3 + 0.12 * events.len() as f64, dur: 0.35, ..Default::default() }));
    events.push(e.clone());
    e
}

const BLOOM: [f64; 4] = [0.3, 0.55, 0.75, 0.9];

fn place(anim: bool, rng: &Rng, layout: &Rng, e: &Event, x: f64, y: f64, f: &dyn Fn()) {
    ink::layer(5);
    let tag = |from: f64, to: Option<f64>| { ink::tag(Some(Tag { event: e.clone(), from, to, overlay: false })); };
    if anim {
        let (r0, l0) = (rng.save(), layout.save());
        for k in 1..=BLOOM.len() {
            tag((k - 1) as f64 / BLOOM.len() as f64, Some(k as f64 / BLOOM.len() as f64));
            ink::push();
            ink::translate(x, y);
            ink::scale(BLOOM[k - 1], None);
            f();
            ink::pop();
            rng.load(r0);
            layout.load(l0);
        }
        tag(1.0, None);
    }
    ink::push();
    ink::translate(x, y);
    f();
    ink::pop();
    if anim { tag(0.0, None) }
}

struct Page {
    anim: bool,
    rng: Rng,
    layout: Rng,
    captions: Vec<Caption>,
    events: Vec<Event>,
    counts: HashMap<&'static str, i32>,
}

impl Page {
    fn figure(&mut self, name: &'static str, x: f64, y: f64, big_r: f64, accent: bool) {
        let f = figures::figure(name).unwrap();
        let p = (f.pick)(&self.layout);
        let e = new_event(&mut self.events);
        let rng = self.rng.clone();
        place(self.anim, &self.rng, &self.layout, &e, x, y, &|| {
            if accent { ink::in_accent(|| (f.draw)(&rng.fork(), big_r, &p)) } else { (f.draw)(&rng.fork(), big_r, &p) }
        });
        *self.counts.entry(name).or_insert(0) += 1;
    }

    /// a symbol in TeX, about 2r tall, a little tilted, in one of the ink's tones or copper
    fn glyph(&mut self, x: f64, y: f64, r: f64) {
        const COLORS: [f64; 5] = [0.06, 0.1, 0.22, 0.35, 0.5];
        let t = GLYPHS[self.layout.int(1.0, GLYPHS.len() as f64) as usize - 1];
        let c = if self.layout.chance(0.15) { ink::accent(0.0) } else { tone(COLORS[self.layout.int(1.0, COLORS.len() as f64) as usize - 1]) };
        let big = if t.contains("displaystyle") { 2.2 } else { 1.0 };
        let rotate = self.layout.sym(18.0);
        let event = new_event(&mut self.events);
        self.captions.push(Caption { x, y, text: format!("${t}$"), anchor: Some("center".into()), small: false,
            scale: Some(r / (0.14 * big)), rotate: Some(rotate), color: Some(c), opacity: Some(0.92), event: Some(event) });
    }

    fn dot(&mut self, x: f64, y: f64, r: f64) {
        let ring = self.layout.chance(0.4);
        let copper = self.layout.chance(0.3);
        let t = self.layout.range(0.3, 0.8);
        let e = new_event(&mut self.events);
        place(self.anim, &self.rng, &self.layout, &e, x, y, &|| {
            let c = if copper { ink::accent(0.0) } else { tone(t) };
            ink::circle(0.0, 0.0, r);
            if ring { ink::stroke(c, 0.5, None, false, None) } else { ink::fill(c, Some(0.85), true) }
        });
    }
}

/// Called by grow.page with the page (g). The TeX captions go back in layout cm.
pub fn page(g: &G) -> Result<PageResult, String> {
    let rng = Rng::new(g.seed * 7919.0 + 41.0);
    let layout = rng.fork();
    ink::set_transform(ink::page_transform(if g.mirror { Some(g.pw) } else { None }));
    let d = g.density.unwrap_or(1.0);
    let anim = g.anim;
    if anim { ink::record(true) }
    let rg = grow::regions(g, &layout);
    let (col, mut foot, spill) = (rg.col, rg.foot, rg.spill);
    // the band along the bottom runs further than the garden's footer
    foot = [foot[0], foot[1], g.textleft + (g.textright - g.textleft) * 0.72, foot[3]];
    let mut regions = vec![col, foot];
    if let Some(s) = spill { regions.push(s) }
    let mut fld = grow::field(&regions);
    fld.disc(g.colright - 0.5, g.headrule + 0.1, 0.7, 1.0); // the header's fleuron
    let fwd = if g.mirror { -1.0 } else { 1.0 };
    let mut pg = Page { anim, rng: rng.clone(), layout: layout.clone(), captions: vec![], events: vec![], counts: HashMap::new() };
    for n in FIGURE_ORDER { pg.counts.insert(n, 0); }

    // 1. the corner figure, where the column meets the bottom
    let cw = col[2] - col[0];
    {
        let big_r = (2.1f64).min(cw * 0.36) * (0.7 + 0.3 * d);
        let (x, y) = (col[0] + big_r + 0.15, col[1] + big_r + 0.1);
        let name = ["gasket", "koch", "vogel", "polyhedron", "chords"][layout.int(1.0, 5.0) as usize - 1];
        if fld.free_at(x, y) >= big_r * 0.9 {
            if name == "koch" {
                let p = [layout.range(0.0, 120.0), 4.0];
                let e = new_event(&mut pg.events);
                let kf = figures::figure("koch").unwrap();
                place(anim, &rng, &layout, &e, x, y, &|| (kf.draw)(&rng.fork(), big_r, &p));
            } else {
                pg.figure(FIGURE_ORDER.iter().copied().find(|n| *n == name).unwrap(), x, y, big_r, false);
            }
            fld.disc(x, y, big_r, 0.95);
        }
    }

    // marks: a small copper figure beside each heading; \florilegiumbloom[name] a figure
    for mk in &g.marks {
        let (heading, arg) = match &mk.kind { MarkKind::Heading(a) => (true, a.to_string()), MarkKind::Bloom(s) => (false, s.clone()) };
        let big_r = if heading { if arg == "1" { 0.42 } else { 0.32 } } else { 0.6 };
        let (x, y) = (g.colright - big_r - 0.25, mk.y + 0.12);
        if fld.free_at(x, y) >= big_r + 0.05 {
            let name = if heading { "hypotrochoid" } else {
                FIGURE_ORDER.iter().copied().find(|n| *n == arg).unwrap_or("maurer")
            };
            pg.figure(name, x, y, big_r, heading);
            fld.disc(x, y, big_r + 0.1, 1.0);
        }
    }

    // 2. families flowing down the column toward the corner, each figure larger than the one
    // before, joined by small copper arrows
    let nfam = if d >= 0.5 { 2 } else if d >= 0.15 { 1 } else { 0 };
    for f in 1..=nfam {
        let name = FAMILY_ORDER[layout.int(1.0, FAMILY_ORDER.len() as f64) as usize - 1];
        let fam = figures::family(name).unwrap();
        let nv = fam.values.len();
        let k = (nv as f64).min(layout.int(3.0, 5.0)) as usize;
        let first = layout.int(1.0, (nv - k + 1) as f64) as usize;
        let p = (fam.pick)(&layout);
        let r0 = layout.range(0.25, 0.38);
        let grow_by = layout.range(1.15, 1.3);
        let sway = layout.range(0.2, 0.6);
        let ph = layout.range(0.0, 2.0 * PI);
        let top = col[3] - 0.4 - (f - 1) as f64 * (col[3] - col[1]) * layout.range(0.3, 0.45);
        let x0 = col[0] + cw * layout.range(0.3, 0.7);
        // the highest start in this stretch where the whole run fits
        let mut best: Option<Vec<[f64; 3]>> = None;
        let mut y0 = top;
        while y0 >= col[1] + 4.0 {
            let (mut pts, mut y, mut r, mut ok) = (vec![], y0, r0, true);
            for i in 1..=k {
                let x = x0 + sway * (ph + 1.3 * i as f64).sin();
                if fld.free_at(x, y) < r + 0.12 { ok = false; break }
                pts.push([x, y, r]);
                y = y - r - 0.45 - r * grow_by;
                r *= grow_by;
            }
            if ok { best = Some(pts); break }
            y0 += -0.2;
        }
        if let Some(best) = best {
            let bow = layout.sym(0.08);
            for (i, c) in best.iter().enumerate() {
                let v = &fam.values[first + i - 1];
                let e = new_event(&mut pg.events);
                let (rngc, cr) = (rng.clone(), c[2]);
                place(anim, &rng, &layout, &e, c[0], c[1], &|| (fam.draw)(&rngc.fork(), cr, v, &p));
                if i > 0 {
                    let a = best[i - 1];
                    let (mut dx, mut dy) = (c[0] - a[0], c[1] - a[1]);
                    let l = (dx * dx + dy * dy).sqrt();
                    dx /= l;
                    dy /= l;
                    let (x0a, y0a, x1, y1) = (a[0] + dx * (a[2] + 0.08), a[1] + dy * (a[2] + 0.08), c[0] - dx * (c[2] + 0.08), c[1] - dy * (c[2] + 0.08));
                    let (mx, my) = ((x0a + x1) / 2.0 - dy * bow, (y0a + y1) / 2.0 + dx * bow);
                    ink::layer(4);
                    if anim { ink::tag(Some(Tag { event: e.clone(), from: 0.0, to: None, overlay: false })); }
                    ink::moveto(x0a, y0a);
                    ink::curveto(mx, my, mx, my, x1, y1);
                    ink::stroke(ink::accent(0.0), 0.3, Some(0.9), false, None);
                    let (mut tx, mut ty) = (x1 - mx, y1 - my);
                    let tl = (tx * tx + ty * ty).sqrt();
                    tx /= tl;
                    ty /= tl;
                    ink::poly(&[[x1, y1], [x1 - tx * 0.09 - ty * 0.035, y1 - ty * 0.09 + tx * 0.035], [x1 - tx * 0.09 + ty * 0.035, y1 - ty * 0.09 - tx * 0.035]], true);
                    ink::fill(ink::accent(0.0), None, false);
                    if anim { ink::tag(None); }
                }
                fld.disc(c[0], c[1], c[2] + 0.08, 1.0);
            }
        }
    }

    // 3. equations along the bottom and in the space under the text, in reading order
    let mut used = [false; 23];
    let mut line = |pg: &mut Page, fld: &mut grow::Field, region: [f64; 4], y: f64, mut scale: f64, x_from: f64| {
        // equations one after another along y, as far as they fit, smaller as they go
        let mut x = x_from;
        let stop = if fwd > 0.0 { region[2] - 0.2 } else { region[0] + 0.2 };
        let h = 0.42 * scale;
        while (stop - x) * fwd > 1.0 {
            let maxw = (stop - x) * fwd;
            let mut t: Option<&str> = None;
            for _ in 0..8 {
                let i = pg.layout.int(1.0, EQUATIONS.len() as f64) as usize - 1;
                if !used[i] && width(EQUATIONS[i]) * scale <= maxw { used[i] = true; t = Some(EQUATIONS[i]); break }
            }
            let Some(t) = t else { break };
            let w = width(t) * scale;
            let x1 = x + fwd * w;
            let mut ok = true;
            let mut u = 0.0;
            while u <= 1.001 {
                if fld.free_at(x + fwd * w * u, y) < h * 0.4 { ok = false; break }
                u += 0.25;
            }
            if ok {
                let color = tone(pg.layout.range(0.06, 0.25));
                let event = new_event(&mut pg.events);
                pg.captions.push(Caption { x, y, text: format!("${t}$"), anchor: Some(if fwd > 0.0 { "west" } else { "east" }.into()),
                    small: false, scale: Some(scale), rotate: None, color: Some(color), opacity: None, event: Some(event) });
                let mut u = 0.0;
                while u <= 1.001 {
                    fld.disc(x + fwd * w * u, y, h * 0.7, 1.0);
                    u += 0.1;
                }
                x = x1 + fwd * pg.layout.range(0.5, 1.0);
            } else {
                x += fwd * 0.6;
            }
            scale *= 0.92;
        }
    };
    if d >= 0.1 {
        let fy = (foot[1] + foot[3]) / 2.0;
        let start = if fwd > 0.0 { foot[0] + 0.4 } else { foot[2] - 0.4 };
        line(&mut pg, &mut fld, foot, fy, 1.15, start);
    }
    if let Some(spill) = spill {
        let rows = 1.0f64.max(((spill[3] - spill[1]) / 4.5 * d + 0.5).floor()) as i64;
        for r in 1..=rows {
            let y = spill[3] - 0.5 - (spill[3] - spill[1] - 0.8) * (r as f64 - 0.5) / rows as f64 + layout.sym(0.3);
            let start = if fwd > 0.0 { spill[0] + layout.range(0.3, 2.5) } else { spill[2] - layout.range(0.3, 2.5) };
            let sc = layout.range(1.0, 1.35);
            line(&mut pg, &mut fld, spill, y, sc, start);
        }
    }

    // 4. the rest, largest hole first: a figure, a symbol or a dot, overlapping a little so
    // that they blend
    let mut area = 0.0;
    for r in &regions { area += (r[2] - r[0]) * (r[3] - r[1]) }
    let nmax = (d * area / 2.8 + 0.5).floor();
    let minf = 0.16 / d.sqrt();
    let mut n = 0.0;
    while n < nmax {
        let (mut best, mut bi): (f64, Option<usize>) = (-1.0, None);
        for i in 0..fld.x.len() {
            let v = fld.free[i] * layout.range(0.85, 1.15);
            if v > best { best = v; bi = Some(i) }
        }
        let Some(bi) = bi else { break };
        if fld.free[bi] < minf { break }
        let (mut f, x, y) = (fld.free[bi], fld.x[bi], fld.y[bi]);
        // larger toward the corner, smaller up the column and out along the bottom
        let reach = (y - col[1]) / (col[3] - col[1]) + 0.0f64.max((x - col[2]) * fwd) / 1.0f64.max(foot[2] - foot[0]);
        f *= 1.1 - 0.5 * 1.0f64.min(reach);
        let u = layout.u();
        if f > 0.55 && u < 0.45 {
            // a figure, not one there are already two of
            let mut name: Option<&'static str> = None;
            for _ in 0..6 {
                let c = FIGURE_ORDER[layout.int(1.0, FIGURE_ORDER.len() as f64) as usize - 1];
                if pg.counts[c] < 2 { name = Some(c); break }
            }
            let big_r = (f * 0.95).min(1.25);
            match name {
                Some(nm) if big_r >= figures::figure(nm).unwrap().r[0] * 0.6 => {
                    pg.figure(nm, x, y, big_r, false);
                    fld.disc(x, y, big_r, 0.62);
                }
                _ => {
                    pg.glyph(x, y, f.min(0.6));
                    fld.disc(x, y, f.min(0.6), 0.5);
                }
            }
        } else if f > 0.22 && u < 0.88 {
            let r = f.min(0.6) * layout.range(0.6, 1.0);
            pg.glyph(x, y, r);
            fld.disc(x, y, r, 0.5);
        } else {
            let r = f.min(0.25) * layout.range(0.3, 0.6);
            pg.dot(x, y, r);
            fld.disc(x, y, r.max(0.15), 0.8);
        }
        n += 1.0;
    }

    let captions = pg.captions;
    if anim {
        let mut total = 0.0f64;
        for e in &pg.events { let e = e.borrow(); total = total.max(e.t0 + e.dur) }
        return Ok(PageResult { rec: Some(ink::take_recording()), total: total.max(0.5), captions, ..Default::default() });
    }
    ink::flush();
    Ok(PageResult { captions, ..Default::default() })
}

// ---------------------------------------------------------------- every figure on its own

fn first_pick(f: &figures::Figure) -> usize { (f.pick)(&Rng::new(1.0)).len() }

fn index_ok(x: f64, n: usize) -> bool { x.fract() == 0.0 && x >= 1.0 && x <= n as f64 }

fn finite(xs: &[f64]) -> bool { xs.iter().all(|x| x.is_finite()) }

/// `M.flora[name](rng, R, o)`: a figure or family by name with `o.p` / `o.v`; what the Lua
/// would fail on (a short, non-finite or out-of-range list) is an error here, not a panic.
pub fn draw(name: &str, rng: &Rng, big_r: f64, o: &Opts) -> Result<(), String> {
    let bad = |what: &str| format!("florilegium: math {name}: bad {what}");
    if !big_r.is_finite() { return Err(bad("size")) }
    if name == "hedera" {
        figures::compass(rng, big_r, o.color);
        return Ok(());
    }
    if let Some(f) = figures::figure(name) {
        let p = match &o.p {
            Some(p) => {
                if p.len() < first_pick(f) || !finite(p) { return Err(bad("p")) }
                if name == "polyhedron" && !index_ok(p[0], figures::solids().len()) { return Err(bad("p")) }
                if name == "knot" && !index_ok(p[0], 6) { return Err(bad("p")) }
                p.clone()
            }
            None => (f.pick)(rng),
        };
        (f.draw)(rng, big_r, &p);
        return Ok(());
    }
    if let Some(f) = figures::family(name) {
        let v = match &o.v { Some(v) => v.clone(), None => f.values[0].clone() };
        let p = match &o.p { Some(p) => p.clone(), None => (f.pick)(rng) };
        let need_v = if matches!(name, "cube" | "dodeca") { 2 } else { f.values[0].len() };
        if v.len() < need_v || !finite(&v) { return Err(bad("v")) }
        if p.len() < (f.pick)(&Rng::new(1.0)).len() || !finite(&p) { return Err(bad("p")) }
        if name == "knots" && !index_ok(v[1], 6) { return Err(bad("v")) }
        (f.draw)(rng, big_r, &v, &p);
        return Ok(());
    }
    Err(format!("florilegium: math has no primitive '{name}'"))
}

fn named(name: &str, rng: &Rng, s: f64, o: &Opts) -> Option<Result<(), String>> {
    if name == "hedera" || figures::figure(name).is_some() || figures::family(name).is_some() { Some(draw(name, rng, s, o)) } else { None }
}

fn theme_flora(f: &mut Flora) {
    let names = std::iter::once("hedera").chain(FIGURE_ORDER.iter().copied()).chain(FAMILY_ORDER.iter().copied());
    for name in names {
        f.set(name, GPrim::Sized(Rc::new(move |rng, s, o| { let _ = draw(name, rng, s, o); })));
    }
}

pub fn theme_def() -> ThemeDef {
    ThemeDef { flora: theme_flora, motifs: None, page: Some(page), named: Some(named) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(p: Option<Vec<f64>>, v: Option<Vec<f64>>) -> Opts { Opts { p, v, ..Default::default() } }

    #[test]
    fn named_entry_surfaces_errors() {
        let r = Rng::new(1.0);
        let d = |n: &str, o: &Opts| grow::draw_named(Some("math"), n, &r, 1.0, o);
        assert!(d("knot", &opts(Some(vec![9.0, 0.0]), None)).is_err());
        assert!(d("knot", &opts(Some(vec![1.0]), None)).is_err());
        assert!(d("polyhedron", &opts(Some(vec![f64::NAN, 0.0, 0.0]), None)).is_err());
        assert!(d("knots", &opts(None, Some(vec![3.0, 99.0]))).is_err());
        assert!(d("cube", &opts(None, Some(vec![0.0]))).is_err());
        assert!(d("nosuch", &Opts::default()).is_err());
        assert!(d("knot", &opts(Some(vec![2.0, 30.0]), None)).is_ok());
        assert!(d("flower", &Opts::default()).is_ok());
        assert!(grow::draw_named(None, "nosuch", &r, 1.0, &Opts::default()).is_err());
    }

    #[test]
    fn truncation_values_not_listed_draw() {
        let r = Rng::new(1.0);
        for name in ["cube", "dodeca"] {
            for v in [[0.0, 0.3], [1.0, 0.9], [2.0, 0.5], [3.0, 1.0 / 3.0], [0.0, 0.0], [0.0, 1.0], [7.0, 0.2], [0.0, -1.0], [0.0, 5.0]] {
                assert!(draw(name, &r, 1.0, &opts(None, Some(v.to_vec()))).is_ok(), "{name} {v:?}");
            }
        }
        ink::take_output();
    }
}
