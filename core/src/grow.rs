// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! grow.lua: the grown composition of florilegium.
//!
//! A plant, not a packing. Everything grows from one root in the bottom-left corner of
//! the page: long stems sway up the margin column, leaves hang at their nodes, and a
//! greedy loop in the spirit of Wong, Zongker & Salesin, "Computer-Generated Floral
//! Ornament" (SIGGRAPH 1998) takes the largest free disc within reach of a stem and grows
//! a branch to it, until only small holes are left. Deterministic for a given seed.
//!
//! # Themes
//!
//! A theme is a `Theme { flora, motifs, page }`. [`theme`] builds one from a [`ThemeDef`]
//! and caches it, as the Lua caches `themes[name]`. The hook for the theme modules
//! (Tasks 7 and 8) is [`theme_def`]: add one match arm per theme there, returning a
//! `ThemeDef`:
//!
//! * `flora`: fills the theme's [`Flora`] (primitives by name in front of flora's, and
//!   optionally `stem`), the Lua's `t.flora`;
//! * `motifs`: the Lua's `t.motifs(rows, F)`, given copies of the base rows and the
//!   finished [`Flora`], returning the rows to use. Row `draw` closures receive the
//!   theme's `&Flora` when called, so the copied base rows draw with the theme's
//!   primitives, as the Lua's module-level `F` makes them;
//! * `page`: a theme that is not a plant brings its own composition, which [`page`]
//!   hands the page to (the Lua's `t.page(g, grow)`; the theme module calls
//!   [`regions`], [`field`], [`layers`] of this module as `grow` offers them).
use crate::flora::{self, Opts, Prim};
use crate::ink::{self, Color, Event, EventData, Kind, Phase, Recording, Rng, Tag};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::f64::consts::PI;
use std::rc::Rc;

type Pt = [f64; 2];

const RAD: f64 = PI / 180.0;

fn dist(ax: f64, ay: f64, bx: f64, by: f64) -> f64 { let dx = ax - bx; let dy = ay - by; (dx * dx + dy * dy).sqrt() }
fn lerp(a: f64, b: f64, t: f64) -> f64 { a + (b - a) * t }
fn smoothstep(a: f64, b: f64, x: f64) -> f64 { let t = 1f64.min(0f64.max((x - a) / (b - a))); t * t * (3.0 - 2.0 * t) }
fn norm(x: f64, y: f64) -> (f64, f64) { let l = (x * x + y * y).sqrt(); if l < 1e-9 { (0.0, 1.0) } else { (x / l, y / l) } }
fn deg(x: f64, y: f64) -> f64 { y.atan2(x) / RAD }
fn tone(t: f64) -> Color { ink::tone(t) }

// layers, back to front (the overlay only in the growth animation)
pub const L_GHOST: i32 = 1;
pub const L_STEM: i32 = 2;
pub const L_LEAF: i32 = 3;
pub const L_AIRY: i32 = 4;
pub const L_BLOOM: i32 = 5;
pub const L_ACCENT: i32 = 6;
pub const L_OVERLAY: i32 = 7;

const GRID: f64 = 0.2;
const REACH: f64 = 3.0;
const SMALLEST: f64 = 0.36;
const FILLMIN: f64 = 0.17;

const GROWSTEP: f64 = 0.3;
const BLOOM: [f64; 6] = [0.25, 0.45, 0.62, 0.76, 0.87, 0.95];
const FLYSTEPS: usize = 14;

/// `M.layers`
pub struct Layers { pub ghost: i32, pub stem: i32, pub leaf: i32, pub airy: i32, pub bloom: i32, pub accent: i32 }
pub fn layers() -> Layers {
    Layers { ghost: L_GHOST, stem: L_STEM, leaf: L_LEAF, airy: L_AIRY, bloom: L_BLOOM, accent: L_ACCENT }
}

// ---------------------------------------------------------------- the page table
pub enum MarkKind { Heading(i32), Bloom(String) }
pub struct Mark { pub kind: MarkKind, pub y: f64 }

/// The page table `g`. `spill` is 0 or 1; `mirror` is Lua's `g.mirror` truthiness.
#[derive(Default)]
pub struct G {
    pub pw: f64, pub ph: f64,
    pub textleft: f64, pub textright: f64,
    pub ytop: f64, pub ybot: f64,
    pub colright: f64, pub headrule: f64, pub footy: f64,
    pub mirror: bool,
    pub seed: f64,
    pub spill: i32, pub spilly: f64,
    pub age: Option<f64>,
    pub marks: Vec<Mark>,
    pub visitors: Option<bool>,
    pub theme: Option<String>,
    pub density: Option<f64>,
    pub anim: bool,
}

pub struct Caption {
    pub x: f64, pub y: f64,
    pub text: String,
    pub anchor: Option<String>,
    pub small: bool,
    pub scale: Option<f64>,
    pub rotate: Option<f64>,
    pub color: Option<Color>,
    pub opacity: Option<f64>,
    pub event: Option<Event>,
}

/// what grew, for a page that describes its own garden
pub struct Stats { pub motifs: i64, pub counts: BTreeMap<String, i64>, pub stems: usize, pub cells: usize }

#[derive(Default)]
pub struct PageResult {
    pub captions: Vec<Caption>,
    pub stats: Option<Stats>,
    pub rec: Option<Recording>,
    pub total: f64,
}

// ---------------------------------------------------------------- the theme's primitives
pub type StemFn = Rc<dyn Fn(&[Pt], f64, f64, Option<Color>, Option<f64>, Option<f64>)>;

/// a primitive that a theme brings: `Scaled` ones are drawn after `ink.scale(s)` and take
/// only the options (leaf, leaflet, petal), `Sized` ones take their size
#[derive(Clone)]
pub enum GPrim {
    Scaled(Rc<dyn Fn(&Rng, &Opts)>),
    Sized(Rc<dyn Fn(&Rng, f64, &Opts)>),
}

/// The Lua's `F`: a theme's primitives by name with flora's behind them.
#[derive(Clone, Default)]
pub struct Flora {
    prims: HashMap<String, GPrim>,
    stem: Option<StemFn>,
}

impl Flora {
    pub fn set(&mut self, name: &str, p: GPrim) { self.prims.insert(name.to_string(), p); }
    pub fn set_stem(&mut self, f: StemFn) { self.stem = Some(f); }
    pub fn sized(&self, name: &str, rng: &Rng, s: f64, o: &Opts) {
        match self.prims.get(name) {
            Some(GPrim::Sized(f)) => f(rng, s, o),
            Some(_) => {}
            None => if let Some(Prim::Sized(f)) = flora::prim(name) { f(rng, s, o) },
        }
    }
    pub fn scaled(&self, name: &str, rng: &Rng, o: &Opts) {
        match self.prims.get(name) {
            Some(GPrim::Scaled(f)) => f(rng, o),
            Some(_) => {}
            None => if let Some(Prim::Scaled(f)) = flora::prim(name) { f(rng, o) },
        }
    }
    pub fn draw_named(&self, name: &str, rng: &Rng, s: f64, o: &Opts) -> Result<(), String> {
        let scaled = |f: &dyn Fn()| { ink::push(); ink::scale(s, None); f(); ink::pop(); };
        match self.prims.get(name) {
            Some(GPrim::Sized(f)) => { f(rng, s, o); return Ok(()) }
            Some(GPrim::Scaled(f)) => { scaled(&|| f(rng, o)); return Ok(()) }
            None => {}
        }
        match flora::prim(name) {
            Some(Prim::Sized(f)) => { f(rng, s, o); Ok(()) }
            Some(Prim::Scaled(f)) => { scaled(&|| f(rng, o)); Ok(()) }
            None => Err(format!("florilegium: no primitive '{name}'")),
        }
    }
    pub fn stem(&self, pts: &[Pt], w0: f64, w1: f64, c: Option<Color>, op: Option<f64>, tip: Option<f64>) {
        match &self.stem { Some(f) => f(pts, w0, w1, c, op, tip), None => flora::stem(pts, w0, w1, c, op, tip) }
    }
}

// ---------------------------------------------------------------- motifs
/// r(s): proxy radius; off(s): distance from the disc center back to the primitive's origin,
/// along the motif's axis; upright: the axis leans upward; draw(F, rng, s, p): the primitive
/// in a frame whose +y is the axis (p: progress of its flight in the animation, None on the page).
pub type DrawFn = Rc<dyn Fn(&Flora, &Rng, f64, Option<f64>)>;

#[derive(Clone)]
pub struct Motif {
    pub name: String,
    pub w: f64,
    pub smin: f64,
    pub smax: f64,
    pub max: f64,
    pub hard: bool,
    pub layer: i32,
    pub fly: bool,
    pub hang: bool,
    pub upright: Option<f64>,
    pub spring: Option<f64>,
    pub summer: Option<f64>,
    pub autumn: Option<f64>,
    pub r: Rc<dyn Fn(f64) -> f64>,
    pub off: Rc<dyn Fn(f64) -> f64>,
    pub draw: DrawFn,
}

impl Motif {
    /// a row with the Lua's defaults: nothing hard, flying or hanging, no season weights
    pub fn new(name: &str, w: f64, smin: f64, smax: f64, max: f64, layer: i32,
               r: impl Fn(f64) -> f64 + 'static, off: impl Fn(f64) -> f64 + 'static, draw: DrawFn) -> Motif {
        Motif { name: name.to_string(), w, smin, smax, max, hard: false, layer, fly: false, hang: false,
                upright: None, spring: None, summer: None, autumn: None, r: Rc::new(r), off: Rc::new(off), draw }
    }
}

fn plain(name: &'static str) -> DrawFn {
    Rc::new(move |f, rng, s, _| f.sized(name, rng, s, &Opts::default()))
}

fn base_motifs() -> Vec<Motif> {
    let sin = |a: f64| a.sin();
    let cos = |a: f64| a.cos();
    let mut rows = vec![];

    let mut m = Motif::new("flower", 3.0, 0.75, 1.3, 3.0, L_BLOOM, |s| 1.0 * s, |_| 0.0, plain("flower"));
    m.hard = true; m.spring = Some(0.12); m.autumn = Some(0.45);
    rows.push(m);

    let mut m = Motif::new("lily", 2.0, 0.8, 1.25, 2.0, L_BLOOM, |s| 0.9 * s, |s| 0.15 * s, plain("lily"));
    m.hard = true; m.upright = Some(0.4); m.spring = Some(0.1); m.autumn = Some(0.4);
    rows.push(m);

    let mut m = Motif::new("blossoms", 3.0, 0.55, 0.85, 5.0, L_BLOOM, |s| s, |s| 0.6 * s, Rc::new(move |f, rng, s, _| {
        let n = rng.int(2.0, 4.0);
        let mut k = 1.0;
        while k <= n {
            let a = (k - (n + 1.0) / 2.0) * 38.0 + rng.sym(10.0);
            let d = s * rng.range(0.45, 0.8) * (if k == 1.0 { 1.0 } else { rng.range(0.85, 1.1) });
            let (x, y) = (d * sin(a * RAD), d * cos(a * RAD));
            ink::moveto(0.0, 0.0);
            ink::curveto(0.0, d * 0.4, x * 0.6, y * 0.7, x, y);
            ink::stroke(tone(0.1), 0.35, None, false, None);
            ink::push();
            ink::translate(x, y);
            f.sized("blossom", rng, s * rng.range(0.38, 0.5), &Opts::default());
            ink::pop();
            k += 1.0;
        }
        if rng.chance(0.6) {
            ink::push();
            ink::rotate(rng.sym(50.0));
            ink::scale(0.5 * s, None);
            f.sized("bud", rng, 1.0, &Opts::default());
            ink::pop();
        }
    }));
    m.upright = Some(0.5); m.spring = Some(0.5); m.autumn = Some(0.5);
    rows.push(m);

    let mut m = Motif::new("cyme", 3.0, 1.0, 2.4, 6.0, L_AIRY, |s| 0.45 * s, |s| 0.5 * s, plain("cyme"));
    m.upright = Some(0.6); m.spring = Some(1.3); m.autumn = Some(0.7);
    rows.push(m);

    let mut m = Motif::new("sprig", 2.0, 1.2, 2.8, 4.0, L_AIRY, |s| 0.42 * s, |s| 0.5 * s, plain("sprig"));
    m.upright = Some(0.8); m.spring = Some(1.2);
    rows.push(m);

    let mut m = Motif::new("umbel", 1.5, 0.8, 1.5, 3.0, L_AIRY, |s| 0.75 * s, |s| 0.6 * s, plain("umbel"));
    m.hard = true; m.upright = Some(1.0); m.spring = Some(0.8); m.autumn = Some(0.8);
    rows.push(m);

    let mut m = Motif::new("leaf", 2.0, 0.9, 2.0, 8.0, L_LEAF, |s| 0.32 * s, |s| 0.5 * s, Rc::new(|f, rng, s, _| {
        ink::scale(s, None);
        let w = rng.range(0.22, 0.3);
        let depth = if rng.chance(0.6) { rng.range(0.4, 0.8) } else { 0.0 };
        f.scaled("leaf", rng, &Opts { w: Some(w), depth: Some(depth), ..Opts::default() });
    }));
    m.upright = Some(0.2); m.spring = Some(1.3); m.autumn = Some(1.2);
    rows.push(m);

    let mut m = Motif::new("bud", 1.0, 0.45, 0.75, 5.0, L_BLOOM, |s| 0.45 * s, |s| 0.5 * s, plain("bud"));
    m.upright = Some(0.5); m.spring = Some(3.5); m.autumn = Some(0.3);
    rows.push(m);

    let mut m = Motif::new("bells", 0.7, 0.6, 0.95, 1.0, L_BLOOM, |s| 0.62 * s, |_| 0.0, Rc::new(|f, rng, s, _| {
        f.sized("bell", rng, s, &Opts::default());
        ink::moveto(0.0, 0.0);
        ink::curveto(0.15 * s, 0.25 * s, 0.4 * s, 0.25 * s, 0.5 * s, 0.1 * s);
        ink::stroke(tone(0.1), 0.35, None, false, None);
        ink::push();
        ink::translate(0.5 * s, 0.1 * s);
        f.sized("bell", rng, 0.75 * s, &Opts::default());
        ink::pop();
    }));
    m.hard = true; m.hang = true; m.spring = Some(0.5); m.autumn = Some(0.6);
    rows.push(m);

    let mut m = Motif::new("tendril", 0.6, 0.3, 0.5, 4.0, L_AIRY, |s| 1.0 * s, |s| 1.0 * s, Rc::new(|f, rng, s, _| {
        ink::rotate(90.0);
        f.sized("tendril", rng, s, &Opts::default());
    }));
    m.spring = Some(2.0); m.autumn = Some(1.3);
    rows.push(m);

    let mut m = Motif::new("budding", 3.0, 0.55, 0.9, 6.0, L_BLOOM, |s| 0.9 * s, |s| 0.6 * s, Rc::new(move |f, rng, s, _| {
        let n = rng.int(2.0, 4.0);
        let mut k = 1.0;
        while k <= n {
            let a = (k - (n + 1.0) / 2.0) * 34.0 + rng.sym(10.0);
            let d = s * rng.range(0.5, 0.85);
            let (x, y) = (d * sin(a * RAD), d * cos(a * RAD));
            ink::moveto(0.0, 0.0);
            ink::curveto(0.0, d * 0.4, x * 0.6, y * 0.7, x, y);
            ink::stroke(tone(0.1), 0.35, None, false, None);
            ink::push();
            ink::translate(x, y);
            ink::rotate(-a * 0.6);
            f.sized("bud", rng, s * rng.range(0.45, 0.6), &Opts::default());
            ink::pop();
            k += 1.0;
        }
    }));
    m.upright = Some(0.6); m.spring = Some(1.0); m.summer = Some(0.0); m.autumn = Some(0.0);
    rows.push(m);

    let mut m = Motif::new("seedhead", 2.0, 0.45, 0.75, 3.0, L_AIRY, |s| 1.05 * s, |_| 0.0, plain("seedhead"));
    m.hard = true; m.upright = Some(0.6); m.spring = Some(0.0); m.summer = Some(0.0); m.autumn = Some(1.0);
    rows.push(m);

    let mut m = Motif::new("butterfly", 0.8, 1.1, 1.5, 2.0, L_ACCENT, |s| 0.5 * s, |_| 0.0, Rc::new(|f, rng, s, p| {
        let o = match p {
            Some(p) => Opts { open: Some(0.3 + 0.7 * (p * 5.0 * PI).cos().abs()), ..Opts::default() },
            None => Opts::default(),
        };
        f.sized("butterfly", rng, s, &o);
    }));
    m.hard = true; m.fly = true;
    rows.push(m);

    let mut m = Motif::new("bee", 0.8, 0.7, 0.85, 2.0, L_ACCENT, |s| 0.6 * s, |_| 0.0, Rc::new(|f, rng, s, p| {
        f.sized("bee", rng, s, &Opts { trail: Some(p.is_none()), ..Opts::default() });
    }));
    m.hard = true; m.fly = true;
    rows.push(m);
    rows
}

// ---------------------------------------------------------------- themes
pub type PageFn = fn(&G) -> Result<PageResult, String>;

/// a theme's own by-name drawing with errors: None when the name is not the theme's
pub type NamedFn = fn(&str, &Rng, f64, &Opts) -> Option<Result<(), String>>;

pub struct Theme {
    pub flora: Flora,
    pub motifs: Vec<Motif>,
    pub page: Option<PageFn>,
    pub named: Option<NamedFn>,
}

/// What a theme module brings (see the module docs).
pub struct ThemeDef {
    pub flora: fn(&mut Flora),
    pub motifs: Option<fn(Vec<Motif>, &Flora) -> Vec<Motif>>,
    pub page: Option<PageFn>,
    pub named: Option<NamedFn>,
}

/// The registry of the themes other than flora. Tasks 7 and 8 add their arms:
/// `"geometric" => Some(crate::geometric::theme_def())`, `"math" => Some(crate::math::theme_def())`.
fn theme_def(name: &str) -> Option<ThemeDef> {
    match name {
        "geometric" => Some(crate::geometric::theme_def()),
        "math" => Some(crate::math::theme_def()),
        _ => None,
    }
}

thread_local! {
    static BASE: RefCell<Option<Rc<Theme>>> = const { RefCell::new(None) };
    static THEMES: RefCell<HashMap<String, Rc<Theme>>> = RefCell::new(HashMap::new());
    static CURRENT: RefCell<Option<Rc<Theme>>> = const { RefCell::new(None) };
}

fn base() -> Rc<Theme> {
    BASE.with(|b| b.borrow_mut().get_or_insert_with(|| Rc::new(Theme { flora: Flora::default(), motifs: base_motifs(), page: None, named: None })).clone())
}

/// `M.theme`: select the theme for the next page. None, "" and "flora" are the garden as it
/// always was; an unknown name is the Lua's error.
pub fn theme(name: Option<&str>) -> Result<(), String> {
    let th = match name {
        None | Some("") | Some("flora") => base(),
        Some(name) => {
            if let Some(t) = THEMES.with(|t| t.borrow().get(name).cloned()) { t } else {
                let def = theme_def(name).ok_or_else(|| format!(
                    "florilegium: no theme '{name}' (theme-{name}.lua): module 'theme-{name}' not found"))?;
                let mut tf = Flora::default();
                (def.flora)(&mut tf);
                let rows = base().motifs.clone();
                let motifs = match def.motifs { Some(f) => f(rows, &tf), None => rows };
                let t = Rc::new(Theme { flora: tf, motifs, page: def.page, named: def.named });
                THEMES.with(|m| m.borrow_mut().insert(name.to_string(), t.clone()));
                t
            }
        }
    };
    CURRENT.with(|c| *c.borrow_mut() = Some(th));
    Ok(())
}

/// A primitive drawn by name in a theme (`\florilegiumdraw`): selects the theme, then draws
/// through its own entry point, the theme's primitives or flora's. Unknown names and
/// unusable options are errors, not silence. Scaled primitives (leaf, leaflet, petal) are
/// drawn after `ink.scale(size)`.
pub fn draw_named(theme_name: Option<&str>, name: &str, rng: &Rng, size: f64, o: &Opts) -> Result<(), String> {
    theme(theme_name)?;
    let th = current();
    if let Some(r) = th.named.and_then(|f| f(name, rng, size, o)) { return r }
    th.flora.draw_named(name, rng, size, o)
}

/// the theme in force (flora until `theme` is called)
pub fn current() -> Rc<Theme> {
    CURRENT.with(|c| c.borrow().clone()).unwrap_or_else(base)
}

/// `M.motifs[name]` of the current theme; the last row of that name
pub fn motif(name: &str) -> Option<Motif> {
    current().motifs.iter().rev().find(|m| m.name == name).cloned()
}

/// A fresh Lua process: the theme cache emptied, flora selected.
pub fn reset() {
    THEMES.with(|t| t.borrow_mut().clear());
    CURRENT.with(|c| *c.borrow_mut() = None);
}

// ---------------------------------------------------------------- region
pub type Rect = [f64; 4];

fn inside(r: &Rect, x: f64, y: f64) -> bool { x > r[0] && x < r[2] && y > r[1] && y < r[3] }

fn border(rects: &[Rect], x: f64, y: f64) -> f64 {
    let mut best = -1.0;
    for (ri, r) in rects.iter().enumerate() {
        if inside(r, x, y) {
            let mut d = f64::INFINITY;
            let edges = [[x - r[0], r[0] - 0.05, y], [r[2] - x, r[2] + 0.05, y], [y - r[1], x, r[1] - 0.05], [r[3] - y, x, r[3] + 0.05]];
            for e in &edges {
                let mut open = false;
                for (oi, o) in rects.iter().enumerate() { if oi != ri && inside(o, e[1], e[2]) { open = true } }
                if !open && e[0] < d { d = e[0] }
            }
            if d > best { best = d }
        }
    }
    best
}

pub struct Regions { pub rects: Vec<Rect>, pub col: Rect, pub foot: Rect, pub spill: Option<Rect>, pub fx1: f64 }

/// the regions a garden may use, in page cm: the margin column, the footer band (its length
/// drawn from `layout`) and, on the last page, the space under the text
pub fn regions(g: &G, layout: &Rng) -> Regions {
    let col = [0.3, 0.45, g.colright, g.ph - 0.5];
    let fx1 = g.textleft + (g.textright - g.textleft) * layout.range(0.42, 0.52);
    let foot = [g.colright - 0.01, 0.4, fx1, 1.75f64.min(g.footy - 0.45)];
    let mut rects = vec![col, foot];
    let mut spill = None;
    if g.spill == 1 && (g.spilly - 0.8) - (g.ybot + 0.6) > 2.5 {
        let s = [g.colright - 0.01, g.ybot + 0.6, g.textright - 0.2, g.spilly - 0.8];
        rects.push(s);
        spill = Some(s);
    }
    Regions { rects, col, foot, spill, fx1 }
}

/// the free space: points on a GRID lattice over the regions (x, y), and for each the radius
/// of the largest disc there that stays inside them and clear of every proxy so far (free)
pub struct Field {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub free: Vec<f64>,
    lat: HashMap<i64, usize>,
    maxfree: f64,
}

fn key(ix: i64, iy: i64) -> i64 { ix * 100000 + iy }

pub fn field(rects: &[Rect]) -> Field {
    let (mut gx, mut gy, mut free) = (vec![], vec![], vec![]);
    let mut lat = HashMap::new();
    for r in rects {
        for ix in (r[0] / GRID).floor() as i64..=(r[2] / GRID).floor() as i64 {
            for iy in (r[1] / GRID).floor() as i64..=(r[3] / GRID).floor() as i64 {
                let (x, y) = ((ix as f64 + 0.5) * GRID, (iy as f64 + 0.5) * GRID);
                let k = key(ix, iy);
                if !lat.contains_key(&k) {
                    let b = border(rects, x, y);
                    if b > 0.0 {
                        lat.insert(k, gx.len());
                        gx.push(x);
                        gy.push(y);
                        free.push(b);
                    }
                }
            }
        }
    }
    let mut maxfree = 0.0;
    for &f in &free { if f > maxfree { maxfree = f } }
    Field { x: gx, y: gy, free, lat, maxfree }
}

impl Field {
    pub fn each_near(&self, x: f64, y: f64, rr: f64, f: &mut dyn FnMut(usize)) {
        for ix in ((x - rr) / GRID).floor() as i64..=((x + rr) / GRID).floor() as i64 {
            for iy in ((y - rr) / GRID).floor() as i64..=((y + rr) / GRID).floor() as i64 {
                if let Some(&i) = self.lat.get(&key(ix, iy)) { f(i) }
            }
        }
    }
    /// a proxy disc: cells count it at k of its radius (k < 1 lets things overlap)
    pub fn disc(&mut self, x: f64, y: f64, r: f64, k: f64) {
        let mut near = vec![];
        self.each_near(x, y, r * k + self.maxfree, &mut |i| near.push(i));
        for i in near {
            let d = dist(self.x[i], self.y[i], x, y) - k * r;
            if d < self.free[i] { self.free[i] = d }
        }
    }
    pub fn free_at(&self, x: f64, y: f64) -> f64 {
        match self.lat.get(&key((x / GRID).floor() as i64, (y / GRID).floor() as i64)) { Some(&i) => self.free[i], None => -1.0 }
    }
}

// ---------------------------------------------------------------- animation timing
const T_START: f64 = 0.4;
const T_STEMS: f64 = 3.6;
const T_BRANCHES: f64 = 6.0;
const T_EASE: f64 = 0.55;
const T_FILL: f64 = 1.2;
const T_ACCENT: f64 = 0.5;

fn schedule(events: &[Event]) -> Result<f64, String> {
    let mut lmax = 0f64;
    let (mut nbranch, mut nfill, mut naccent, mut nfall) = (0usize, 0usize, 0usize, 0usize);
    for e in events {
        let e = e.borrow();
        if e.phase == Some(Phase::Stems) { lmax = lmax.max(e.len.unwrap_or(0.0)) }
        match e.phase {
            Some(Phase::Branch) => nbranch += 1,
            Some(Phase::Fill) => nfill += 1,
            Some(Phase::Accent) => naccent += 1,
            Some(Phase::Fall) => nfall += 1,
            _ => {}
        }
    }
    let speed = lmax / T_STEMS;
    let start = |i: f64, n: usize| T_BRANCHES * ((i - 1.0) / 1f64.max(n as f64)).powf(T_EASE);
    for e in events {
        let mut e = e.borrow_mut();
        if let Some(sp) = e.span { e.dur = sp }
        else if e.kind == Kind::Stem { e.dur = e.len.unwrap_or(0.0) / speed }
        else if e.kind == Kind::Bloom {
            e.dur = if e.root_phase == Some(Phase::Ghost) { 1.2 } else { 0.2 + 0.25 * 1f64.min(e.size.unwrap_or(0.5)) }
        } else if e.kind == Kind::Overlay {
            let i = e.parent.as_ref().map(|p| p.borrow().i).unwrap_or(0) as f64;
            e.lead = 0.08f64.max(0.4f64.min(1.5 * (start(i + 1.0, nbranch) - start(i, nbranch))));
        }
        e.dur = e.dur.max(1e-3);
    }
    let resolve = |root_t0: &dyn Fn(&EventData) -> Option<f64>| -> f64 {
        let mut last = 0f64;
        for e in events {
            let mut e = e.borrow_mut();
            if e.t0.is_nan() {
                match e.parent.clone() {
                    None => if let Some(t) = root_t0(&e) { e.t0 = t },
                    Some(p) => {
                        let p = p.borrow();
                        if !p.t0.is_nan() && e.kind == Kind::Overlay {
                            e.t0 = p.t0 - e.lead;
                            e.dur = e.lead + p.dur;
                        } else if !p.t0.is_nan() {
                            e.t0 = p.t0 + e.at.unwrap_or(0.0) * p.dur;
                        }
                    }
                }
            }
            if !e.t0.is_nan() && !e.till_end { last = last.max(e.t0 + e.dur) }
        }
        last
    };
    let t1 = resolve(&|e| match e.phase {
        Some(Phase::Stems) => Some(T_START + e.off.unwrap_or(0.0)),
        Some(Phase::Ghost) => Some(T_START + 0.8 * e.i as f64),
        _ => None,
    });
    let t2 = resolve(&|e| match e.phase {
        Some(Phase::Marks) => Some(t1 + 0.1 + 0.3 * (e.i as f64 - 1.0)),
        Some(Phase::Branch) => Some(t1 + 0.2 + start(e.i as f64, nbranch)),
        _ => None,
    });
    let t3 = resolve(&|e| match e.phase {
        Some(Phase::Fall) => Some(t2 - 0.4 + T_FILL * (e.i as f64 - 1.0) / 1f64.max(nfall as f64)),
        Some(Phase::Fill) => Some(t2 + 0.1 + T_FILL * (e.i as f64 - 1.0) / 1f64.max(nfill as f64)),
        _ => None,
    });
    let t4 = resolve(&|e| match e.phase {
        Some(Phase::Tail) => Some(t3),
        Some(Phase::Accent) => Some(t3 + T_ACCENT * (e.i as f64 - 1.0) / 1f64.max(naccent as f64)),
        _ => None,
    });
    for e in events {
        let mut e = e.borrow_mut();
        if e.t0.is_nan() { return Err("florilegium: an animation event without a start".into()) }
        if e.till_end { e.dur = 0.5f64.max(t4 - e.t0) }
    }
    Ok(t4)
}

/// the part [0,t] of a cubic Bézier, by de Casteljau
#[allow(clippy::too_many_arguments)]
fn bezier_head(x0: f64, y0: f64, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, t: f64) -> [f64; 6] {
    let (ax, ay) = (lerp(x0, x1, t), lerp(y0, y1, t));
    let (bx, by) = (lerp(x1, x2, t), lerp(y1, y2, t));
    let (cx, cy) = (lerp(x2, x3, t), lerp(y2, y3, t));
    let (dx, dy) = (lerp(ax, bx, t), lerp(ay, by, t));
    let (ex, ey) = (lerp(bx, cx, t), lerp(by, cy, t));
    [ax, ay, dx, dy, lerp(dx, ex, t), lerp(dy, ey, t)]
}

// ---------------------------------------------------------------- page
type StemPt = [f64; 5];

struct Leafy { pts: Vec<Pt>, big: bool, ev: Option<Event> }

struct Garden {
    anim: bool,
    events: Vec<Event>,
    now: Option<Event>,
    rng: Rng,
    layout: Rng,
    th: Rc<Theme>,
    fld: Field,
    sdist: Vec<f64>,
    spts: Vec<StemPt>,
}

fn event_of(kind: Kind, phase: Option<Phase>) -> EventData { EventData { kind, phase, t0: f64::NAN, ..Default::default() } }

/// the 1-based tangent of a polyline at point i
fn tangent(pts: &[Pt], i: usize) -> (f64, f64) {
    let n = pts.len();
    let (a, b) = (pts[i.saturating_sub(1).max(1) - 1], pts[(i + 1).min(n) - 1]);
    norm(b[0] - a[0], b[1] - a[1])
}

impl Garden {
    /// each event's output is tagged { event, from, to }, visible while the event's progress
    /// is in [from, to)
    fn event(&mut self, mut d: EventData) -> Option<Event> {
        if !self.anim { return None; }
        d.t0 = f64::NAN;
        d.root_phase = match &d.parent { Some(p) => p.borrow().root_phase, None => d.phase };
        let e = Rc::new(RefCell::new(d));
        self.events.push(e.clone());
        self.now = Some(e.clone());
        ink::tag(Some(Tag { event: e.clone(), from: 0.0, to: None, overlay: false }));
        Some(e)
    }

    fn tag(&self, from: f64, to: Option<f64>) {
        if let Some(e) = &self.now { ink::tag(Some(Tag { event: e.clone(), from, to, overlay: false })); }
    }

    fn add_stem_points(&mut self, pts: &[Pt], w0: f64, w1: f64, every: usize) {
        let n = pts.len();
        let mut i = 1;
        while i <= n {
            let (a, b) = (pts[i.saturating_sub(1).max(1) - 1], pts[(i + 1).min(n) - 1]);
            let (tx, ty) = norm(b[0] - a[0], b[1] - a[1]);
            let p = pts[i - 1];
            self.spts.push([p[0], p[1], tx, ty, lerp(w0, w1, (i - 1) as f64 / 1f64.max(n as f64 - 1.0))]);
            let (fld, sdist) = (&self.fld, &mut self.sdist);
            fld.each_near(p[0], p[1], REACH, &mut |j| {
                let d = dist(fld.x[j], fld.y[j], p[0], p[1]);
                if d < sdist[j] { sdist[j] = d }
            });
            i += every;
        }
    }

    fn nearest_stem(&self, x: f64, y: f64) -> Result<(StemPt, f64), String> {
        let (mut best, mut bp) = (f64::INFINITY, None);
        for p in &self.spts {
            let d = dist(x, y, p[0], p[1]);
            if d < best { best = d; bp = Some(*p) }
        }
        bp.map(|p| (p, best)).ok_or_else(|| "florilegium: no stem to grow from".to_string())
    }

    fn stem(&mut self, pts: &[Pt], w0: f64, w1: f64, proxy: Option<f64>) {
        ink::layer(L_STEM);
        let f = &self.th.flora;
        if self.anim {
            // growing: the first k/n of its length, for progress in [k/n, (k+1)/n)
            let (s, l) = ink::arclen(pts);
            let n = 1i64.max((l / GROWSTEP).floor() as i64);
            let mut j = 1usize;
            if let Some(now) = &self.now { now.borrow_mut().len = Some(l) }
            for k in 1..n {
                let at = l * k as f64 / n as f64;
                while s[j] < at { j += 1 }
                let d = s[j] - s[j - 1];
                let u = if d > 1e-9 { (at - s[j - 1]) / d } else { 0.0 };
                let mut head: Vec<Pt> = pts[..j].to_vec();
                head.push([lerp(pts[j - 1][0], pts[j][0], u), lerp(pts[j - 1][1], pts[j][1], u)]);
                self.tag(k as f64 / n as f64, Some((k + 1) as f64 / n as f64));
                f.stem(&head, w0, lerp(w0, w1, k as f64 / n as f64), None, None, Some(0.4));
            }
            self.tag(1.0, None);
        }
        f.stem(pts, w0, w1, None, None, None);
        if self.anim { self.tag(0.0, None) }
        let proxy = proxy.unwrap_or(0.1);
        let mut i = 0;
        while i < pts.len() { self.fld.disc(pts[i][0], pts[i][1], proxy, 1.0); i += 2 }
    }

    /// draw a primitive at (x,y) with its +y axis at angle `a` (degrees from +x). In the
    /// animation it opens, or with path(e) -> x, y, a it travels to its place in `steps`
    /// positions and f gets the progress e.
    #[allow(clippy::too_many_arguments)]
    fn place(&self, layer: i32, x: f64, y: f64, a: f64, f: &dyn Fn(Option<f64>),
             path: Option<&dyn Fn(f64) -> (f64, f64, f64)>, steps: Option<usize>) {
        ink::layer(layer);
        if self.anim {
            // a few earlier copies with the same random draws, then the real one
            let (r0, l0) = (self.rng.save(), self.layout.save());
            let n = if path.is_some() { steps.unwrap_or(FLYSTEPS) } else { BLOOM.len() };
            for k in 1..=n {
                self.tag((k - 1) as f64 / n as f64, Some(k as f64 / n as f64));
                ink::push();
                if let Some(path) = path {
                    let (px, py, pa) = path((k - 1) as f64 / n as f64);
                    ink::translate(px, py);
                    ink::rotate(pa - 90.0);
                    f(Some((k - 1) as f64 / n as f64));
                } else {
                    ink::translate(x, y);
                    ink::rotate(a - 90.0);
                    ink::scale(BLOOM[k - 1], None);
                    f(None);
                }
                ink::pop();
                self.rng.load(r0);
                self.layout.load(l0);
            }
            self.tag(1.0, None);
        }
        ink::push();
        ink::translate(x, y);
        ink::rotate(a - 90.0);
        f(None);
        ink::pop();
        if self.anim { self.tag(0.0, None) }
    }

    /// the explained animation marks the free disc a branch grows to, and its stem point
    fn overlay(&mut self, parent: &Option<Event>, x: f64, y: f64, r: f64, s: &StemPt) {
        if !self.anim { return; }
        let Some(parent) = parent else { return };
        let mut d = event_of(Kind::Overlay, None);
        d.parent = Some(parent.clone());
        d.root_phase = parent.borrow().root_phase;
        let e = Rc::new(RefCell::new(d));
        self.events.push(e.clone());
        let old = ink::tag(Some(Tag { event: e, from: 0.0, to: Some(1.0), overlay: true }));
        ink::layer(L_OVERLAY);
        ink::circle(x, y, r);
        ink::stroke(ink::accent(0.0), 0.7, Some(0.9), false, Some("[2.5 2] 0"));
        ink::circle(x, y, 0.035);
        ink::fill(ink::accent(0.0), None, false);
        ink::circle(s[0], s[1], 0.05);
        ink::fill(ink::accent(0.0), None, false);
        ink::tag(old);
    }

    /// a branch from stem point S to a motif at (bx,by) whose axis is (ax,ay)
    #[allow(clippy::too_many_arguments)]
    fn branch_to(&mut self, s: &StemPt, bx: f64, by: f64, ax: f64, ay: f64, w0: f64, ev: &Option<Event>) -> Vec<Pt> {
        let th = self.th.clone();
        let pts = flora::branch_path(&self.layout, s[0], s[1], s[2], s[3], bx, by, ax, ay);
        let d = dist(s[0], s[1], bx, by);
        let w = (s[4] * 0.7).min(w0);
        self.stem(&pts, w, 0.012f64.max(w * 0.45), Some(0.07));
        self.add_stem_points(&pts, w, 0.012f64.max(w * 0.45), 2);
        if d > 1.0 {
            let nl = if d > 2.0 { 2 } else { 1 };
            for j in 1..=nl {
                let i = (pts.len() as f64 * (0.25 + 0.3 * j as f64 / nl as f64)).floor() as usize;
                let (p, q) = (pts[i - 1], pts[i]);
                let a = deg(q[0] - p[0], q[1] - p[1]) + (if j % 2 == 0 { 1.0 } else { -1.0 }) * self.layout.range(40.0, 60.0);
                let ls = self.layout.range(0.45, 0.75);
                self.event(EventData { parent: ev.clone(), at: Some(i as f64 / pts.len() as f64), size: Some(ls), ..event_of(Kind::Bloom, None) });
                let rng = self.rng.clone();
                self.place(L_LEAF, p[0], p[1], a, &|_| {
                    ink::scale(ls, None);
                    th.flora.scaled("leaflet", &rng.fork(), &Opts::default());
                }, None, None);
                self.fld.disc(p[0] + (a * RAD).cos() * ls * 0.5, p[1] + (a * RAD).sin() * ls * 0.5, 0.25 * ls, 0.7);
            }
        }
        pts
    }
}

/// a flight path for the animation: from distance d in direction `from` (degrees), easing
/// out, swinging sideways by `swing`, wobbling about the final heading
fn arrive(x: f64, y: f64, a: f64, d: f64, from: f64, swing: f64) -> impl Fn(f64) -> (f64, f64, f64) {
    let (ux, uy) = ((from * RAD).cos(), (from * RAD).sin());
    move |e| {
        let q = 1.0 - (1.0 - e) * (1.0 - e);
        let w = swing * (PI * q).sin();
        (x + d * ux * (1.0 - q) - uy * w, y + d * uy * (1.0 - q) + ux * w, a + 25.0 * (3.0 * PI * e).sin() * (1.0 - q))
    }
}

struct Visiting { butterfly: bool, bee: bool, snail: bool, bird: bool }
impl Visiting {
    fn get(&self, name: &str) -> bool {
        match name { "butterfly" => self.butterfly, "bee" => self.bee, "snail" => self.snail, "bird" => self.bird, _ => false }
    }
}

/// With g.anim set, the page is recorded instead of written, and the recording is returned
/// with its events timed. g.density (0..1, default 1) thins the garden: the loops stop at
/// larger holes, proxies keep more room around them, fewer leaves and fillers.
pub fn page(g: &G) -> Result<PageResult, String> {
    theme(g.theme.as_deref())?;
    let th = current();
    if let Some(composition) = th.page { return composition(g); }
    let d_ = g.density.unwrap_or(1.0);
    let nstem: usize = if d_ >= 0.6 { 3 } else if d_ >= 0.25 { 2 } else { 1 };
    let footer = d_ >= 0.2;
    let rng = Rng::new(g.seed * 7919.0 + 13.0);
    let layout = rng.fork();
    ink::set_transform(ink::page_transform(if g.mirror { Some(g.pw) } else { None }));

    let anim = g.anim;
    if anim { ink::record(true) }
    let vis = Rng::new(g.seed * 7919.0 + 101.0);
    let mut visiting = if g.visitors == Some(false) {
        Visiting { butterfly: false, bee: false, snail: false, bird: false }
    } else {
        let butterfly = vis.chance(0.75);
        let bee = vis.chance(0.6);
        let snail = vis.chance(0.4);
        let bird = vis.chance(0.22);
        Visiting { butterfly, bee, snail, bird }
    };
    if d_ < 0.3 { visiting.snail = false; visiting.bird = false }
    let age = g.age;
    let season = |sp: f64, su: f64, au: f64| -> f64 {
        match age {
            None => su,
            Some(age) if age < 0.5 => lerp(sp, su, smoothstep(0.1, 0.5, age)),
            Some(age) => lerp(su, au, smoothstep(0.5, 0.9, age)),
        }
    };
    let spring = season(1.0, 0.0, 0.0);
    let autumn = season(0.0, 0.0, 1.0);
    let sea = Rng::new(g.seed * 7919.0 + 211.0);
    let mut blooms: Vec<[f64; 3]> = vec![];

    let Regions { rects, col: _, foot, spill, fx1 } = regions(g, &layout);
    let fld = field(&rects);
    let sdist = vec![f64::INFINITY; fld.x.len()];
    let mut gd = Garden { anim, events: vec![], now: None, rng: rng.clone(), layout: layout.clone(), th: th.clone(), fld, sdist, spts: vec![] };
    let f = &th.flora;
    let motifs = &th.motifs;

    // ---------------------------------------------------------- 1. stems
    let root = [layout.range(1.0, 1.6), 0.75];
    let cx = g.colright * 0.47;
    let amp = g.colright * 0.2;
    let mut mains: Vec<Vec<Pt>> = vec![];
    let mut mainev: Vec<Option<Event>> = vec![];
    let tops = {
        let t1 = g.ph - 2.6;
        let t2 = g.ph * layout.range(0.6, 0.72);
        let t3 = g.ph * layout.range(0.36, 0.48);
        [t1, t2, t3]
    };
    for k in 1..=nstem {
        let kf = k as f64;
        let fb = ink::fbm1(&layout, 2);
        let ph = layout.range(0.0, 10.0);
        let lam = layout.range(6.0, 9.0);
        let x0 = root[0] + (kf - 2.0) * 0.25;
        let mut pts = vec![];
        let mut y = root[1];
        while y <= tops[k - 1] {
            let mut x = cx + amp * fb(y / lam + ph) + (kf - 2.0) * 0.35;
            x = lerp(x0, x, smoothstep(0.0, 4.0, y - root[1]));
            x = 0.9f64.max((g.colright - 0.9).min(x));
            pts.push([x, y]);
            y += 0.25;
        }
        let ev = gd.event(event_of(Kind::Stem, Some(Phase::Stems)));
        let w0 = if k == 1 { 0.085 } else { 0.065 };
        gd.stem(&pts, w0, 0.028, None);
        gd.add_stem_points(&pts, w0, 0.028, 1);
        mains.push(pts);
        mainev.push(ev);
    }
    // the scroll: the root curls into a spiral at the corner
    {
        let mut pts = vec![];
        let turns = layout.range(1.2, 1.6);
        for i in 0..=40 {
            let t = i as f64 / 40.0;
            let a = PI * 0.5 + t * turns * 2.0 * PI;
            let r = 0.55 * (1.0 - 0.85 * t).powf(1.3);
            pts.push([root[0] - 0.55 + r * a.cos() * -1.0, root[1] + 0.05 + r * a.sin()]);
        }
        gd.event(EventData { off: Some(0.3), ..event_of(Kind::Stem, Some(Phase::Stems)) });
        gd.stem(&pts, 0.06, 0.012, None);
    }
    // the footer stem: from the root along the bottom, tapering out to a hairline
    let mut footpts: Vec<Pt> = vec![];
    let mut footev = None;
    if footer {
        let fb = ink::fbm1(&layout, 2);
        let ybase = (foot[1] + foot[3]) / 2.0 - 0.05;
        let mut x = root[0];
        while x <= fx1 - 0.3 {
            let u = smoothstep(root[0], root[0] + 1.5, x);
            let y = lerp(root[1] + 0.1, ybase + 0.22 * fb(x / 3.5), u);
            footpts.push([x, y]);
            x += 0.2;
        }
        footev = gd.event(EventData { off: Some(0.5), ..event_of(Kind::Stem, Some(Phase::Stems)) });
        gd.stem(&footpts, 0.065, 0.01, Some(0.06));
        gd.add_stem_points(&footpts, 0.065, 0.012, 1);
    }
    // on the last page, branches arch out of the main stems into the space under the text
    let mut leafy: Vec<Leafy> = (0..nstem).map(|k| Leafy { pts: mains[k].clone(), big: true, ev: mainev[k].clone() }).collect();
    let mut arcs: Vec<Vec<Pt>> = vec![];
    let mut arcev: Vec<Option<Event>> = vec![];
    if let Some(spill) = spill {
        let sh = spill[3] - spill[1];
        let mut nb = 1f64.max((sh / 7.0).floor());
        if d_ < 1.0 { nb = (nb * d_ + 0.5).floor() }
        let mut b = 1.0;
        while b <= nb {
            let y0 = spill[1] + sh * (b - 0.75) / nb;
            let si = layout.int(1.0, nstem as f64) as usize - 1;
            let src = &mains[si];
            let (mut best, mut bi) = (f64::INFINITY, 1usize);
            for (i, p) in src.iter().enumerate() {
                let d = (p[1] - y0).abs();
                if d < best { best = d; bi = i + 1 }
            }
            let s0 = src[bi - 1];
            let w = spill[2] - s0[0];
            let x3 = s0[0] + w * layout.range(0.7, 0.88);
            let y3 = (spill[3] - 1.4).min(y0 + sh / nb * layout.range(0.35, 0.6));
            let c1y = s0[1] + layout.range(-0.4, 0.8);
            let c2y = y3 + layout.range(-1.2, -0.3);
            let bz = ink::bezier3(s0[0], s0[1], s0[0] + w * 0.25, c1y, x3 - w * 0.3, c2y, x3, y3);
            let mut pts = ink::sample(&bz, (w / 0.2).floor() as usize, None, None);
            let fb = ink::fbm1(&layout, 2);
            let np = pts.len();
            for (i, p) in pts.iter_mut().enumerate() {
                let u = i as f64 / (np as f64 - 1.0);
                p[1] += 0.35 * fb(p[0] / 3.0) * (PI * u).sin();
            }
            let (sa, la) = ink::arclen(src);
            let e = gd.event(EventData { parent: mainev[si].clone(), at: Some(sa[bi - 1] / la), ..event_of(Kind::Stem, None) });
            gd.stem(&pts, 0.06, 0.025, None);
            gd.add_stem_points(&pts, 0.06, 0.025, 1);
            leafy.push(Leafy { pts: pts.clone(), big: true, ev: e.clone() });
            arcs.push(pts);
            arcev.push(e);
            b += 1.0;
        }
    }

    // the header fleuron keeps its spot free
    gd.fld.disc(g.colright - 0.5, g.headrule + 0.1, 0.7, 1.0);

    // ---------------------------------------------------------- 2. terminal flowers and ghosts
    let mut counts: BTreeMap<String, i64> = BTreeMap::new();
    for m in motifs { counts.insert(m.name.clone(), 0); }
    {
        let m1 = &mains[0];
        let (p, q) = (m1[m1.len() - 1], m1[m1.len() - 3]);
        let (hx, hy) = norm(p[0] - q[0], p[1] - q[1]);
        let s = 1.75f64.min(g.colright * 0.27);
        let (fx, fy) = (p[0] + hx * 0.2, p[1] + hy * 0.2);
        gd.event(EventData { parent: mainev[0].clone(), at: Some(1.0), size: Some(s), ..event_of(Kind::Bloom, None) });
        gd.place(L_BLOOM, fx, fy, 90.0, &|_| f.sized("flower", &rng.fork(), s, &Opts::default()), None, None);
        gd.fld.disc(fx, fy, 1.05 * s, 0.85);
        blooms.push([fx, fy, s]);
        for k in 2..=nstem {
            let pts = &mains[k - 1];
            let (p2, q2) = (pts[pts.len() - 1], pts[pts.len() - 3]);
            let (ax, ay) = norm(p2[0] - q2[0], p2[1] - q2[1]);
            // the shorter stems end in a lily and a flower; in spring in buds, in autumn gone to seed
            let tip = if spring > 0.5 { Some("bud") } else if autumn > 0.5 { Some("seedhead") } else { None };
            if k == 2 {
                let s2 = layout.range(1.0, 1.2);
                gd.event(EventData { parent: mainev[1].clone(), at: Some(1.0), size: Some(s2), ..event_of(Kind::Bloom, None) });
                match tip {
                    Some("bud") => gd.place(L_BLOOM, p2[0], p2[1], deg(ax, ay), &|_| f.sized("bud", &rng.fork(), 0.95 * s2, &Opts::default()), None, None),
                    Some(_) => gd.place(L_AIRY, p2[0] + ax * 0.15, p2[1] + ay * 0.15, 90.0, &|_| f.sized("seedhead", &rng.fork(), 0.75 * s2, &Opts::default()), None, None),
                    None => gd.place(L_BLOOM, p2[0] + ax * 0.25 * s2, p2[1] + ay * 0.25 * s2, deg(ax, ay), &|_| f.sized("lily", &rng.fork(), s2, &Opts::default()), None, None),
                }
                gd.fld.disc(p2[0] + ax * 0.3 * s2, p2[1] + ay * 0.3 * s2, 0.95 * s2, 0.85);
                *counts.entry("lily".into()).or_insert(0) += 1;
            } else {
                let s3 = layout.range(0.85, 1.0);
                gd.event(EventData { parent: mainev[2].clone(), at: Some(1.0), size: Some(s3), ..event_of(Kind::Bloom, None) });
                match tip {
                    Some("bud") => gd.place(L_BLOOM, p2[0], p2[1], deg(ax, ay), &|_| f.sized("bud", &rng.fork(), 0.9 * s3, &Opts::default()), None, None),
                    Some(_) => gd.place(L_AIRY, p2[0], p2[1], 90.0, &|_| f.sized("seedhead", &rng.fork(), 0.75 * s3, &Opts::default()), None, None),
                    None => {
                        gd.place(L_BLOOM, p2[0], p2[1], 90.0, &|_| f.sized("flower", &rng.fork(), s3, &Opts::default()), None, None);
                        blooms.push([p2[0], p2[1], s3]);
                    }
                }
                gd.fld.disc(p2[0], p2[1], s3, 0.85);
                *counts.entry("flower".into()).or_insert(0) += 1;
            }
        }
    }
    for (b, pts) in arcs.iter().enumerate() {
        let p = pts[pts.len() - 1];
        let s = layout.range(1.0, 1.25);
        gd.event(EventData { parent: arcev[b].clone(), at: Some(1.0), size: Some(s), ..event_of(Kind::Bloom, None) });
        gd.place(L_BLOOM, p[0], p[1], 90.0, &|_| f.sized("flower", &rng.fork(), s, &Opts::default()), None, None);
        gd.fld.disc(p[0], p[1], s, 0.85);
        *counts.entry("flower".into()).or_insert(0) += 1;
        blooms.push([p[0], p[1], s]);
    }
    // ghosts: large pale flowers behind, given a little room so they show
    {
        let mut ng = layout.int(1.0, 2.0) as usize;
        if d_ < 0.4 { ng = 0 }
        for k in 1..=ng {
            let gxp = g.colright * layout.range(0.35, 0.65);
            let gyp = g.ph * (if k == 1 { layout.range(0.15, 0.3) } else { layout.range(0.45, 0.6) });
            let s = layout.range(1.3, 1.6);
            gd.event(EventData { i: k, ..event_of(Kind::Bloom, Some(Phase::Ghost)) });
            gd.place(L_GHOST, gxp, gyp, 90.0, &|_| flora::ghost(Some(0.62), || f.sized("flower", &rng.fork(), s, &Opts::default())), None, None);
            gd.fld.disc(gxp, gyp, 0.8 * s, 0.6);
        }
    }

    // ---------------------------------------------------------- marks: flowers the text asks for
    let mut nmarks = 0usize;
    for mk in &g.marks {
        let (m, mut s) = match &mk.kind {
            MarkKind::Heading(arg) => (motif(if *arg == 1 { "flower" } else { "blossoms" }), if *arg == 1 { 0.95 } else { 0.7 }),
            MarkKind::Bloom(arg) => {
                let m = motif(if arg.is_empty() { "flower" } else { arg });
                let s = m.as_ref().map(|m| if arg.is_empty() { 0.9 } else { (m.smin + m.smax) / 2.0 }).unwrap_or(0.0);
                (m, s)
            }
        };
        let Some(m) = m else { continue };
        let mut r = (m.r)(s);
        let (px, py) = (g.colright - r - 0.15, mk.y + 0.12);
        let fr = gd.fld.free_at(px, py);
        if fr < 0.95 * r {
            let (mut lo, mut hi) = (m.smin * 0.8, s);
            for _ in 0..20 { let mid = (lo + hi) / 2.0; if (m.r)(mid) > fr * 0.95 { hi = mid } else { lo = mid } }
            s = lo;
            r = (m.r)(s);
        }
        if r <= fr * 1.15 && py > 0.6 + r && py < g.ph - 0.6 - r {
            let (sp, _) = gd.nearest_stem(px, py)?;
            let (dx, dy) = norm(px - sp[0], py - sp[1]);
            let (ax, ay) = norm(dx, dy + m.upright.unwrap_or(0.0));
            let (ox, oy) = (px - ax * (m.off)(s), py - ay * (m.off)(s));
            nmarks += 1;
            let bev = gd.event(EventData { i: nmarks, ..event_of(Kind::Stem, Some(Phase::Marks)) });
            if m.fly {
                gd.event(EventData { parent: bev.clone(), at: Some(1.0), span: Some(1.2), ..event_of(Kind::Bloom, None) });
                let draw = m.draw.clone();
                let path = arrive(px, py, 90.0, 3.0, 200.0, 0.5);
                gd.place(m.layer, px, py, 90.0, &|p| draw(f, &rng.fork(), s, p), Some(&path), None);
            } else {
                gd.branch_to(&sp, ox, oy, ax, ay, 0.045, &bev);
                gd.event(EventData { parent: bev.clone(), at: Some(1.0), size: Some(r), ..event_of(Kind::Bloom, None) });
                // a heading's flower is painted in the accent, as headings were rubricated
                let heading = matches!(mk.kind, MarkKind::Heading(_));
                let draw = m.draw.clone();
                gd.place(m.layer, ox, oy, deg(ax, ay), &|_| {
                    if heading { ink::in_accent(|| draw(f, &rng.fork(), s, None)) } else { draw(f, &rng.fork(), s, None) }
                }, None, None);
                if m.name == "flower" { blooms.push([ox, oy, s]) }
            }
            *counts.entry(m.name.clone()).or_insert(0) += 1;
            gd.fld.disc(px, py, r, 0.85);
        }
    }

    // visitors on the stems: a snail crawling up a main stem, and now and then a bird on a twig of its own
    if visiting.snail {
        let s = vis.range(0.75, 0.9);
        let mut best = -1.0;
        let mut c: Option<(usize, usize, f64, f64, f64)> = None;
        for _ in 0..12 {
            let k = vis.int(1.0, nstem as f64) as usize;
            let pts = &mains[k - 1];
            let n = pts.len() as f64;
            let i = vis.int(10f64.max((n * 0.15).floor()), 11f64.max((n * 0.45).floor())) as usize;
            let side = if vis.chance(0.5) { 1.0 } else { -1.0 };
            let (tx, ty) = tangent(pts, i);
            let (nx, ny) = (ty * side, -tx * side);
            let fr = gd.fld.free_at(pts[i - 1][0] + nx * 0.8 * s, pts[i - 1][1] + ny * 0.8 * s);
            if fr > best { best = fr; c = Some((k, i, side, nx, ny)) }
        }
        if let Some((ck, ci, side, nx, ny)) = c.filter(|_| best > 0.25) {
            let pts = &mains[ck - 1];
            let (sa, la) = ink::arclen(pts);
            let i0 = 1f64.max(ci as f64 - 8.0);
            let at = |e: f64| -> (f64, f64, f64) {
                let j = lerp(i0, ci as f64, e);
                let j0 = j.floor() as usize;
                let j1 = pts.len().min(j0 + 1);
                let (p, q) = (pts[j0 - 1], pts[j1 - 1]);
                let (tx, ty) = tangent(pts, j0);
                let u = j - j0 as f64;
                (lerp(p[0], q[0], u) + nx * 0.03, lerp(p[1], q[1], u) + ny * 0.03, deg(tx, ty))
            };
            let (x, y, a) = at(1.0);
            gd.event(EventData { parent: mainev[ck - 1].clone(), at: Some(sa[ci - 1] / la), till_end: true, ..event_of(Kind::Bloom, None) });
            gd.place(L_ACCENT, x, y, a, &|_| {
                ink::scale(side, Some(1.0));
                f.sized("snail", &rng.fork(), s, &Opts::default());
            }, Some(&at), Some(40));
            gd.fld.disc(pts[ci - 1][0] + nx * 0.36 * s, pts[ci - 1][1] + ny * 0.36 * s, 0.5 * s, 1.0);
            counts.insert("snail".into(), 1);
        }
    }
    if visiting.bird {
        let s = vis.range(2.3, 2.7);
        let mut best = -1.0;
        // k, i, ex, ey, dir, facing, cx, cy
        let mut c: Option<(usize, usize, f64, f64, f64, f64, f64, f64)> = None;
        for _ in 0..24 {
            let k = vis.int(1.0, nstem as f64) as usize;
            let pts = &mains[k - 1];
            let n = pts.len() as f64;
            let i = vis.int((n * 0.25).floor(), (n * 0.75).floor()) as usize;
            let p = pts[i - 1];
            let dir = if vis.chance(0.5) { 1.0 } else { -1.0 };
            let l = vis.range(1.2, 1.7);
            let (ex, ey) = (p[0] + dir * l, p[1] + vis.range(0.05, 0.4));
            let (bx, by) = (ex - dir * 0.25, ey);
            let facing = if vis.chance(0.65) { dir } else { -dir };
            let (cx, cy) = (bx + facing * 0.03 * s, by + 0.4 * s);
            let fr = (gd.fld.free_at(cx, cy) / (0.36 * s)).min(gd.fld.free_at(bx - facing * 0.35 * s, by + 0.15 * s) / (0.2 * s));
            if fr > best { best = fr; c = Some((k, i, ex, ey, dir, facing, cx, cy)) }
        }
        if let Some((ck, ci, ex, ey, dir, facing, cx, cy)) = c.filter(|_| best > 1.0) {
            let pts = &mains[ck - 1];
            let p = pts[ci - 1];
            let (tx, ty) = tangent(pts, ci);
            let (sa, la) = ink::arclen(pts);
            let twig = flora::branch_path(&vis, p[0], p[1], tx, ty, ex, ey, dir, 0.1);
            let tev = gd.event(EventData { parent: mainev[ck - 1].clone(), at: Some(sa[ci - 1] / la), ..event_of(Kind::Stem, None) });
            gd.stem(&twig, 0.05, 0.02, Some(0.07));
            gd.add_stem_points(&twig, 0.05, 0.02, 2);
            let perch = twig[1.max((twig.len() as f64 * 0.85).floor() as usize) - 1];
            gd.event(EventData { parent: tev.clone(), at: Some(1.0), span: Some(1.4), ..event_of(Kind::Bloom, None) });
            let path = arrive(perch[0], perch[1], 90.0, 3.5, if facing > 0.0 { 160.0 } else { 20.0 }, 0.6);
            gd.place(L_ACCENT, perch[0], perch[1], 90.0, &|_| f.sized("bird", &rng.fork(), s, &Opts { facing: Some(facing), ..Opts::default() }),
                Some(&path), Some(FLYSTEPS));
            gd.fld.disc(cx, cy, 0.36 * s, 0.9);
            gd.fld.disc(perch[0] - facing * 0.35 * s, perch[1] + 0.15 * s, 0.2 * s, 0.9);
            counts.insert("bird".into(), 1);
        }
    }

    // ---------------------------------------------------------- 3. leaves at the nodes
    if footer { leafy.push(Leafy { pts: footpts.clone(), big: false, ev: footev.clone() }) }
    for st in &leafy {
        let pts = &st.pts;
        let mut s = 0.0;
        let mut acc = layout.range(0.6, 1.3);
        let mut side = if layout.chance(0.5) { 1.0 } else { -1.0 };
        let mut total = 0.0;
        for i in 1..pts.len() { total += dist(pts[i][0], pts[i][1], pts[i - 1][0], pts[i - 1][1]) }
        // Lua's `for i = 2, #pts - 4` (1-based) is 0-based 1 .. len-5
        let last = pts.len() as i64 - 4;
        let mut i1 = 2i64;
        while i1 <= last {
            let i = (i1 - 1) as usize;
            s += dist(pts[i][0], pts[i][1], pts[i - 1][0], pts[i - 1][1]);
            if s >= acc {
                let u = s / total;
                acc += (if st.big { layout.range(1.4, 2.2) } else { lerp(0.55, 1.4, u) * layout.range(0.8, 1.2) }) / d_.powf(0.6);
                side = -side;
                let (tx, ty) = norm(pts[i + 1][0] - pts[i - 1][0], pts[i + 1][1] - pts[i - 1][1]);
                let a = deg(tx, ty) + side * layout.range(35.0, 60.0);
                let (ux, uy) = ((a * RAD).cos(), (a * RAD).sin());
                let mut lf = if st.big { lerp(2.4, 1.3, pts[i][1] / g.ph) * layout.range(0.85, 1.1) } else { lerp(0.95, 0.45, u) * layout.range(0.85, 1.1) };
                lf *= season(0.8, 1.0, 1.0);
                let mut ok = false;
                for _ in 0..3 {
                    let (mx, my) = (pts[i][0] + ux * lf * 0.55, pts[i][1] + uy * lf * 0.55);
                    if gd.fld.free_at(mx, my) >= 0.24 * lf { ok = true; break }
                    lf *= 0.75;
                }
                if ok && lf > (if st.big { 0.7 } else { 0.3 }) {
                    gd.event(EventData { parent: st.ev.clone(), at: Some(u), size: Some(0.4 * lf), ..event_of(Kind::Bloom, None) });
                    if st.big {
                        let lobed = layout.chance(0.6);
                        let bend = side * layout.range(0.05, 0.3);
                        // young leaves are pale; in autumn some turn copper
                        let copper = autumn > 0.0 && sea.chance(0.35 * autumn);
                        gd.place(L_LEAF, pts[i][0], pts[i][1], a, &|_| {
                            let leaf = || {
                                ink::scale(lf, None);
                                let r = rng.fork();
                                let w = layout.range(0.22, 0.3);
                                let depth = if lobed { layout.range(0.4, 0.8) } else { 0.0 };
                                f.scaled("leaf", &r, &Opts { w: Some(w), depth: Some(depth), bend: Some(bend), ..Opts::default() });
                            };
                            if copper { ink::in_accent(leaf) } else if spring > 0.0 { flora::ghost(Some(0.3 * spring), leaf) } else { leaf() }
                        }, None, None);
                    } else {
                        let bend = -side * layout.range(0.1, 0.3);
                        gd.place(L_LEAF, pts[i][0], pts[i][1], a, &|_| {
                            ink::scale(lf, None);
                            f.scaled("leaflet", &rng.fork(), &Opts { bend: Some(bend), ..Opts::default() });
                        }, None, None);
                    }
                    for fr in [0.3, 0.55, 0.8] { gd.fld.disc(pts[i][0] + ux * lf * fr, pts[i][1] + uy * lf * fr, 0.24 * lf, 0.85) }
                }
            }
            i1 += 1;
        }
    }

    // ---------------------------------------------------------- 4. branches to the largest holes
    let mut area = 0.0;
    for r in &rects { area += (r[2] - r[0]) * (r[3] - r[1]) }
    let af = 1f64.max(area / 175.0);
    let mut caps: HashMap<String, i64> = HashMap::new();
    for m in motifs {
        let c = if m.fly { (m.max * 1f64.min(2.0 * d_) + 0.5).floor() } else { (m.max * af * d_ + 0.5).floor() };
        caps.insert(m.name.clone(), c as i64);
    }
    let mut nbranch = 0usize;
    let nbmax = if d_ < 1.0 { 1f64.max((40.0 * d_).floor()) } else { f64::INFINITY };

    for _ in 0..400 {
        let (mut best, mut bi) = (-1.0, None);
        for i in 0..gd.fld.x.len() {
            if gd.sdist[i] <= REACH {
                let v = gd.fld.free[i] * layout.range(0.85, 1.15);
                if v > best { best = v; bi = Some(i) }
            }
        }
        let Some(bi) = bi else { break };
        if gd.fld.free[bi] < SMALLEST / d_.sqrt() || nbranch as f64 >= nbmax { break }
        let (fr, px, py) = (gd.fld.free[bi], gd.fld.x[bi], gd.fld.y[bi]);
        // statement motifs have a hard cap; the others only lose weight as they repeat
        let (mut elig, mut wts, mut wsum) = (vec![], vec![], 0.0);
        for m in motifs {
            let cnt = counts[&m.name];
            let cap = caps[&m.name];
            let wt = m.w * season(m.spring.unwrap_or(1.0), m.summer.unwrap_or(1.0), m.autumn.unwrap_or(1.0)) / (1.0 + cnt as f64 / 1f64.max(cap as f64));
            let ok = wt > 0.0 && (m.r)(m.smin) <= fr * 1.05 && (!m.hard || cnt < cap) && (!m.fly || visiting.get(&m.name));
            if ok { elig.push(m); wts.push(wt); wsum += wt }
        }
        if elig.is_empty() { gd.fld.free[bi] = 0.0 } else {
            let mut pick = elig[elig.len() - 1];
            let mut acc = layout.u() * wsum;
            for (k, m) in elig.iter().enumerate() {
                acc -= wts[k];
                if acc <= 0.0 { pick = m; break }
            }
            let mut s = pick.smax;
            if (pick.r)(s) > fr * 0.95 {
                let (mut lo, mut hi) = (pick.smin, s);
                for _ in 0..20 { let mid = (lo + hi) / 2.0; if (pick.r)(mid) > fr * 0.95 { hi = mid } else { lo = mid } }
                s = lo;
            }
            let (sp, _sd) = gd.nearest_stem(px, py)?;
            let (dx, dy) = norm(px - sp[0], py - sp[1]);
            // the motif's axis: away from the stem, leaning up for the upright kinds
            let (ax, ay) = norm(dx, dy + pick.upright.unwrap_or(0.0));
            let (mut ox, mut oy) = (px - ax * (pick.off)(s), py - ay * (pick.off)(s));
            if pick.hang { ox = px - dx * 0.2 * s; oy = py + 0.45 * s }
            nbranch += 1;
            let bev = gd.event(EventData { i: nbranch, ..event_of(Kind::Stem, Some(Phase::Branch)) });
            gd.overlay(&bev, px, py, fr, &sp);
            let draw = pick.draw.clone();
            if pick.fly {
                // a visitor: no branch, any heading, and in the animation it flies in
                let a = if pick.name == "bee" { layout.range(0.0, 360.0) } else { 90.0 + layout.sym(50.0) };
                gd.event(EventData { parent: bev.clone(), at: Some(1.0), span: Some(1.2), ..event_of(Kind::Bloom, None) });
                let from = layout.range(0.0, 360.0);
                let swing = layout.sym(0.8);
                let path = arrive(px, py, a, 3.0, from, swing);
                gd.place(pick.layer, px, py, a, &|p| draw(f, &rng.fork(), s, p), Some(&path), None);
            } else {
                if dist(ox, oy, sp[0], sp[1]) > 0.12 {
                    gd.branch_to(&sp, ox, oy, if pick.hang { dx } else { ax }, if pick.hang { -0.3 } else { ay }, 0.04, &bev);
                }
                let a = if pick.hang { 90.0 } else { deg(ax, ay) };
                gd.event(EventData { parent: bev.clone(), at: Some(1.0), size: Some((pick.r)(s)), ..event_of(Kind::Bloom, None) });
                gd.place(pick.layer, ox, oy, a, &|_| draw(f, &rng.fork(), s, None), None, None);
                if pick.name == "flower" { blooms.push([ox, oy, s]) }
            }
            *counts.get_mut(&pick.name).unwrap() += 1;
            gd.fld.disc(px, py, (pick.r)(s) * (2.0 - d_), 0.85);
        }
    }

    // autumn: loose petals fall from the flowers into the free space below them
    if autumn > 0.0 {
        let mut nfall = 0usize;
        for b in &blooms {
            let nf = sea.int(0.0, (3.0 * autumn + 0.5).floor()) as i64;
            for _ in 0..nf {
                let (mut best, mut bx, mut by) = (0.16, None, 0.0);
                for _ in 0..6 {
                    let x = b[0] + sea.sym(1.1);
                    let y = b[1] - b[2] - sea.range(0.1, 2.2);
                    let fr = gd.fld.free_at(x, y);
                    if fr > best { best = fr; bx = Some(x); by = y }
                }
                if let Some(bx) = bx {
                    let ps = sea.range(0.4, 0.6);
                    let pa = sea.range(0.0, 360.0);
                    let pt = sea.range(0.36, 0.5);
                    let sway = sea.sym(0.6);
                    let x0 = b[0] + sea.sym(0.3);
                    let y0 = b[1] - 0.3 * b[2];
                    nfall += 1;
                    gd.event(EventData { i: nfall, span: Some(1.6), ..event_of(Kind::Bloom, Some(Phase::Fall)) });
                    let path = |e: f64| (lerp(x0, bx, e) + sway * (2.0 * PI * e).sin(), lerp(y0, by, e * e * (3.0 - 2.0 * e)), pa - 140.0 * (1.0 - e));
                    gd.place(L_BLOOM, bx, by, pa, &|_| {
                        ink::scale(ps, None);
                        f.scaled("petal", &rng.fork(), &Opts { tone: Some(pt), heart: Some(false), ..Opts::default() });
                    }, Some(&path), Some(16));
                    gd.fld.disc(bx, by, 0.22, 1.0);
                }
            }
        }
    }

    // ---------------------------------------------------------- 5. small holes
    let (mut accents, mut fills) = (0.0f64, 0usize);
    let maxfill = (28.0 * af * d_).floor();
    for _ in 0..600 {
        let (mut best, mut bi) = (-1.0, None);
        for i in 0..gd.fld.x.len() {
            let v = gd.fld.free[i] * layout.range(0.85, 1.15);
            if v > best { best = v; bi = Some(i) }
        }
        let Some(bi) = bi else { break };
        if gd.fld.free[bi] < FILLMIN / d_ { break }
        let (fr, px, py) = (gd.fld.free[bi], gd.fld.x[bi], gd.fld.y[bi]);
        let (sp, sd) = gd.nearest_stem(px, py)?;
        if sd < 0.75 && (fills as f64) < maxfill {
            fills += 1;
            gd.event(EventData { i: fills, size: Some(fr), ..event_of(Kind::Bloom, Some(Phase::Fill)) });
            let (dx, dy) = norm(px - sp[0], py - sp[1]);
            let (ax, ay) = norm(dx, dy + 0.4);
            let kind = layout.u();
            if kind < 0.4 {
                // two leaflets in a V at the stem point
                let base = deg(ax, ay);
                let ls = 0.75f64.min(fr * 1.6);
                for k in [-1.0, 1.0] {
                    let a = base + k * layout.range(20.0, 35.0);
                    gd.place(L_LEAF, sp[0], sp[1], a, &|_| {
                        ink::scale(ls * layout.range(0.8, 1.0), None);
                        f.scaled("leaflet", &rng.fork(), &Opts { bend: Some(-k * 0.15), ..Opts::default() });
                    }, None, None);
                }
            } else if kind < 0.65 {
                let (ox, oy) = (px - ax * fr * 0.5, py - ay * fr * 0.5);
                ink::layer(L_STEM);
                ink::moveto(sp[0], sp[1]);
                ink::curveto(sp[0] + sp[2] * 0.15, sp[1] + sp[3] * 0.15, ox - ax * 0.1, oy - ay * 0.1, ox, oy);
                ink::stroke(tone(0.1), 0.4, None, false, None);
                gd.place(L_BLOOM, ox, oy, deg(ax, ay), &|_| f.sized("bud", &rng.fork(), 0.6f64.min(fr * 1.7), &Opts::default()), None, None);
            } else {
                // a small cluster of florets on hairline stalks
                let n = layout.int(2.0, 4.0) as usize;
                for _ in 1..=n {
                    let qx = px + layout.sym(fr * 0.5);
                    let qy = py + layout.sym(fr * 0.5);
                    ink::layer(L_STEM);
                    ink::moveto(sp[0], sp[1]);
                    ink::curveto(sp[0] + sp[2] * 0.1, sp[1] + sp[3] * 0.1, (sp[0] + qx) / 2.0, (sp[1] + qy) / 2.0 + 0.05, qx, qy);
                    ink::stroke(tone(0.12), 0.25, Some(0.9), false, None);
                    gd.place(L_BLOOM, qx, qy, 90.0, &|_| {
                        let r = rng.fork();
                        f.sized("floret", &r, 0.11f64.min(fr * 0.6) * layout.range(0.8, 1.1), &Opts::default());
                    }, None, None);
                }
            }
        } else if sd >= 0.75 && accents < 6.0 * d_ && fr > 0.22 && px < g.colright {
            accents += 1.0;
            gd.event(EventData { i: accents as usize, size: Some(0.3), ..event_of(Kind::Bloom, Some(Phase::Accent)) });
            if layout.chance(0.6) {
                gd.place(L_ACCENT, px, py, 90.0, &|_| f.sized("sparkle", &rng.fork(), 0.13, &Opts::default()), None, None);
            } else {
                gd.place(L_ACCENT, px, py, 90.0, &|_| f.sized("rosette", &rng.fork(), 0.12, &Opts::default()), None, None);
            }
        }
        gd.fld.disc(px, py, fr.max(0.3), 1.0);
    }

    // ---------------------------------------------------------- 6. the footer's tail
    if footer {
        let e = footpts[footpts.len() - 1];
        let x2 = e[0] + layout.range(1.6, 2.2);
        ink::layer(L_ACCENT);
        let c = [e[0], e[1], e[0] + 0.6, e[1] + 0.05, x2 - 0.6, e[1] - 0.05, x2, e[1]];
        let tail = gd.event(EventData { len: Some(x2 - e[0]), ..event_of(Kind::Stem, Some(Phase::Tail)) });
        if anim {
            for k in 1..=7 {
                gd.tag(k as f64 / 8.0, Some((k + 1) as f64 / 8.0));
                ink::moveto(c[0], c[1]);
                let h = bezier_head(c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7], k as f64 / 8.0);
                ink::curveto(h[0], h[1], h[2], h[3], h[4], h[5]);
                ink::stroke(ink::accent(0.0), 0.3, Some(0.8), false, None);
            }
            gd.tag(1.0, None);
        }
        ink::moveto(c[0], c[1]);
        ink::curveto(c[2], c[3], c[4], c[5], c[6], c[7]);
        ink::stroke(ink::accent(0.0), 0.3, Some(0.8), false, None);
        gd.event(EventData { parent: tail, at: Some(1.0), size: Some(0.3), ..event_of(Kind::Bloom, None) });
        gd.place(L_ACCENT, x2 + 0.18, e[1], 90.0, &|_| f.sized("sparkle", &rng.fork(), 0.12, &Opts { rays: Some(false), ..Opts::default() }), None, None);
        ink::circle(x2 + 0.45, e[1], 0.025);
        ink::fill(ink::accent(0.0), None, false);
    }

    // assemble: a safety clip a little outside the regions, then the layers
    ink::tag(None);
    ink::layer(0);
    for (i, r) in rects.iter().enumerate() {
        let rx = r[2] + if i == 0 { 0.25 } else { 0.3 };
        ink::poly(&[[r[0] - 0.25, r[1] - 0.3], [rx, r[1] - 0.3], [rx, r[3] + 0.3], [r[0] - 0.25, r[3] + 0.3]], true);
    }
    ink::clip();
    ink::layer(99);
    ink::unclip();
    let n: i64 = counts.values().sum();
    if anim {
        let total = schedule(&gd.events)?;
        return Ok(PageResult { rec: Some(ink::take_recording()), total, ..Default::default() });
    }
    ink::flush();
    Ok(PageResult { stats: Some(Stats { motifs: n, counts, stems: gd.spts.len(), cells: gd.fld.x.len() }), ..Default::default() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_theme_is_the_lua_error() {
        let e = theme(Some("nosuch")).unwrap_err();
        assert!(e.starts_with("florilegium: no theme 'nosuch' (theme-nosuch.lua): "), "{e}");
        assert!(theme(None).is_ok() && theme(Some("")).is_ok() && theme(Some("flora")).is_ok());
    }
}
