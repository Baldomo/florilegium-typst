// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! ink.lua: the drawing core. Geometry is computed here and written as PDF path
//! operators; `svg.rs` turns them into SVG.
use crate::fmt::fixed;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::f64::consts::PI;
use std::rc::Rc;

pub type Color = [f64; 3];
pub type Tf = [f64; 6];
type Pt = [f64; 2];

const RAD: f64 = PI / 180.0;
pub const CM: f64 = 72.0 / 2.54;
pub const WHITE: Color = [1.0, 1.0, 1.0];
pub const PAPER: Color = [0.995, 0.993, 0.985];

// ---------------------------------------------------------------- random
// mulberry32 (Tommy Ettinger, 2017, public domain): small, fast, identical on
// every platform
#[derive(Clone)]
pub struct Rng(Rc<Cell<u32>>);

impl Rng {
    pub fn new(seed: f64) -> Rng {
        Rng(Rc::new(Cell::new(seed.floor() as i64 as u32)))
    }
    pub fn u(&self) -> f64 {
        let s = self.0.get().wrapping_add(0x6D2B79F5);
        self.0.set(s);
        let mut t = s;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t = t ^ t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        ((t ^ (t >> 14)) as f64) / 4294967296.0
    }
    pub fn range(&self, a: f64, b: f64) -> f64 { a + (b - a) * self.u() }
    pub fn sym(&self, a: f64) -> f64 { a * (2.0 * self.u() - 1.0) }
    pub fn int(&self, a: f64, b: f64) -> f64 { a + ((b - a + 1.0) * self.u()).floor() }
    pub fn chance(&self, p: f64) -> bool { self.u() < p }
    pub fn pick<T: Clone>(&self, t: &[T]) -> T { t[(t.len() as f64 * self.u()).floor() as usize].clone() }
    pub fn normal(&self) -> f64 {
        let u1 = self.u().max(1e-12);
        (-2.0 * u1.ln()).sqrt() * (2.0 * PI * self.u()).cos()
    }
    pub fn fork(&self) -> Rng { Rng::new((self.u() * 4294967295.0).floor()) }
    pub fn save(&self) -> u32 { self.0.get() }
    pub fn load(&self, v: u32) { self.0.set(v) }
}

// ---------------------------------------------------------------- noise
// 1-D gradient noise with a seeded gradient table, smooth, roughly in [-1,1]
pub fn noise1(rng: &Rng) -> Rc<dyn Fn(f64) -> f64> {
    let g: Vec<f64> = (0..256).map(|_| rng.sym(1.0)).collect();
    Rc::new(move |x| {
        let i = x.floor();
        let f = x - i;
        let i = i as i64;
        let a = g[(i & 255) as usize] * f;
        let b = g[((i + 1) & 255) as usize] * (f - 1.0);
        let u = f * f * (3.0 - 2.0 * f);
        2.0 * (a + (b - a) * u)
    })
}

// fractal sum of 1-D noise: `oct` octaves, each double frequency, half amplitude
pub fn fbm1(rng: &Rng, oct: i32) -> Rc<dyn Fn(f64) -> f64> {
    let ns: Vec<_> = (0..oct).map(|_| noise1(rng)).collect();
    Rc::new(move |x| {
        let (mut v, mut a, mut f) = (0.0, 1.0, 1.0);
        for k in 1..=oct {
            v = v + a * ns[(k - 1) as usize](x * f + 17.0 * k as f64);
            a = a * 0.5;
            f = f * 2.0;
        }
        v
    })
}

// ---------------------------------------------------------------- color
fn srgb_dec(c: f64) -> f64 { if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) } }
fn srgb_enc(c: f64) -> f64 { if c <= 0.0031308 { 12.92 * c } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 } }
fn cbrt(x: f64) -> f64 { if x < 0.0 { -((-x).powf(1.0 / 3.0)) } else { x.powf(1.0 / 3.0) } }

// OKLCH is Oklab in polar form. Oklab and the matrices below are Björn
// Ottosson's ("A perceptual color space for image processing", 2020, public domain).
fn rgb_to_oklch(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (r, g, b) = (srgb_dec(r), srgb_dec(g), srgb_dec(b));
    let l = cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
    let m = cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
    let s = cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
    let ll = 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s;
    let a = 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s;
    let b = 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s;
    (ll, (a * a + b * b).sqrt(), b.atan2(a) / RAD)
}

// OKLCH -> sRGB; None when out of gamut
fn oklch_to_rgb(ll: f64, c: f64, h: f64) -> Option<Color> {
    let (a, b) = (c * (h * RAD).cos(), c * (h * RAD).sin());
    let l = (ll + 0.3963377774 * a + 0.2158037573 * b).powf(3.0);
    let m = (ll - 0.1055613458 * a - 0.0638541728 * b).powf(3.0);
    let s = (ll - 0.0894841775 * a - 1.2914855480 * b).powf(3.0);
    let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;
    if r < -1e-4 || g < -1e-4 || b < -1e-4 || r > 1.0001 || g > 1.0001 || b > 1.0001 { return None; }
    let c = |x: f64| srgb_enc(1f64.min(0f64.max(x)));
    Some([c(r), c(g), c(b)])
}

// largest in-gamut chroma at or below C
fn oklch_clamped(ll: f64, c: f64, h: f64) -> Color {
    if let Some(rgb) = oklch_to_rgb(ll, c, h) { return rgb; }
    let (mut lo, mut hi) = (0.0, c);
    for _ in 0..18 {
        let mid = (lo + hi) / 2.0;
        if oklch_to_rgb(ll, mid, h).is_some() { lo = mid } else { hi = mid }
    }
    oklch_to_rgb(ll, lo, h).unwrap_or([0.0; 3])
}

// t, L, C, hue drift
const STOPS: [[f64; 4]; 6] = [
    [0.00, 0.25, 0.105, 6.0],
    [0.18, 0.35, 0.150, 3.0],
    [0.38, 0.47, 0.175, 0.0],
    [0.58, 0.60, 0.145, -3.0],
    [0.80, 0.77, 0.080, -6.0],
    [1.00, 0.92, 0.030, -8.0],
];
const REF_C: f64 = 0.175;

type Ramp = Rc<RefCell<Vec<Color>>>;

struct State {
    ramp: Ramp,
    hue: f64,
    chroma_k: f64,
    lift: f64,
    pale: f64,
    accent: Color,
    accent_ramp: Option<Ramp>,
    accent_key: Option<String>,
    t: Tf,
    stack: Vec<Tf>,
    layers: BTreeMap<i32, Vec<String>>,
    tags: BTreeMap<i32, Vec<Option<Tag>>>,
    cur: i32,
    tag: Option<Tag>,
    recording: bool,
    path: String,
}

impl State {
    fn new() -> State {
        let mut s = State {
            ramp: Rc::new(RefCell::new(vec![[0.0; 3]; 101])),
            hue: 263.0,
            chroma_k: 1.0,
            lift: 0.0,
            pale: 0.0,
            accent: [0.71, 0.40, 0.18],
            accent_ramp: None,
            accent_key: None,
            t: [CM, 0.0, 0.0, CM, 0.0, 0.0],
            stack: vec![],
            layers: BTreeMap::new(),
            tags: BTreeMap::new(),
            cur: 5,
            tag: None,
            recording: false,
            path: String::new(),
        };
        s.set_ink(0x24 as f64 / 255.0, 0x56 as f64 / 255.0, 0xC4 as f64 / 255.0);
        s
    }

    fn build_ramp(&mut self) {
        for i in 0..=100usize {
            let t = i as f64 / 100.0;
            let mut k = 0;
            while STOPS[k + 1][0] < t { k += 1 }
            let (a, b) = (STOPS[k], STOPS[k + 1]);
            let u = (t - a[0]) / (b[0] - a[0]);
            let ll = a[1] + (b[1] - a[1]) * u;
            let c = (a[2] + (b[2] - a[2]) * u) * self.chroma_k;
            let h = self.hue + a[3] + (b[3] - a[3]) * u;
            self.ramp.borrow_mut()[i] = oklch_clamped(ll, c, h);
        }
    }

    fn set_ink(&mut self, r: f64, g: f64, b: f64) {
        let (_, c, h) = rgb_to_oklch(r, g, b);
        self.hue = h;
        self.chroma_k = 0.15f64.max(1.3f64.min(c / REF_C));
        self.build_ramp();
    }

    fn put(&mut self, s: String) {
        let cur = self.cur;
        let b = self.layers.entry(cur).or_default();
        b.push(s);
        let tag = if self.recording { self.tag.clone() } else { None };
        self.tags.entry(cur).or_default().push(tag);
    }

    fn take(&mut self) -> String {
        std::mem::take(&mut self.path)
    }
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::new());
    static SINK: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R { STATE.with(|s| f(&mut s.borrow_mut())) }

/// A fresh Lua process: all state back to its initial value, the output sink emptied.
pub fn reset() {
    with(|s| *s = State::new());
    SINK.with(|s| s.borrow_mut().clear());
}

/// The literals written since the last call (what node.write received).
pub fn take_output() -> Vec<String> { SINK.with(|s| std::mem::take(&mut *s.borrow_mut())) }

// set the hue from the document color `ink` (r,g,b in 0..1)
pub fn set_ink(r: f64, g: f64, b: f64) { with(|s| s.set_ink(r, g, b)) }
pub fn set_accent(r: f64, g: f64, b: f64) { with(|s| s.accent = [r, g, b]) }

// a color is [r,g,b]; tone(t) is a point on the ramp (ghost lift applied)
pub fn tone(t: f64) -> Color {
    with(|s| {
        let mut t = t + (1.0 - t) * s.lift;
        if s.pale > 0.0 { t = t + (1.0 - t) * s.pale }
        let i = 0f64.max(100f64.min((t * 100.0 + 0.5).floor())) as usize;
        let c = s.ramp.borrow()[i];
        c
    })
}

fn toward_white(c: Color, v: f64) -> Color {
    [c[0] + (1.0 - c[0]) * v, c[1] + (1.0 - c[1]) * v, c[2] + (1.0 - c[2]) * v]
}

/// k>0 lightens toward white, k<0 darkens
pub fn accent(k: f64) -> Color {
    with(|s| {
        let mut c = if k >= 0.0 { toward_white(s.accent, k) }
            else { [s.accent[0] * (1.0 + k), s.accent[1] * (1.0 + k), s.accent[2] * (1.0 + k)] };
        if s.pale > 0.0 { c = toward_white(c, s.pale) }
        c
    })
}

// paint f() with a ramp at the hue of the accent instead (copper leaves in autumn)
pub fn in_accent(f: impl FnOnce()) {
    let main = with(|s| {
        let a = s.accent;
        let main = s.ramp.clone();
        let mut key = String::new();
        for (i, v) in a.iter().enumerate() {
            if i > 0 { key.push(' ') }
            fixed(&mut key, *v, 3);
        }
        if s.accent_key.as_deref() != Some(&key) {
            let (h0, c0) = (s.hue, s.chroma_k);
            let (_, c, h) = rgb_to_oklch(a[0], a[1], a[2]);
            s.ramp = Rc::new(RefCell::new(vec![[0.0; 3]; 101]));
            s.hue = h;
            s.chroma_k = 0.15f64.max(1.3f64.min(c / REF_C));
            s.build_ramp();
            s.accent_ramp = Some(s.ramp.clone());
            s.accent_key = Some(key);
            s.hue = h0;
            s.chroma_k = c0;
        }
        s.ramp = s.accent_ramp.clone().unwrap();
        main
    });
    f();
    with(|s| s.ramp = main);
}

pub fn set_lift(v: f64) -> f64 { with(|s| std::mem::replace(&mut s.lift, v)) }
pub fn get_lift() -> f64 { with(|s| s.lift) }
// paint everything paler: tone t -> t + (1-t)*v, accents mixed toward white as much.
// Only the color: the level of detail still follows the ghost's lift alone.
pub fn set_pale(v: f64) -> f64 { with(|s| std::mem::replace(&mut s.pale, v)) }

// ---------------------------------------------------------------- transforms
// an affine map [a,b,c,d,e,f]: x' = a x + c y + e,  y' = b x + d y + f
pub fn set_transform(t: Tf) { with(|s| s.t = t) }

// the page: cm from the lower-left corner of the paper -> bp. With `mirror` (the
// paper width in cm) x becomes mirror - x
pub fn page_transform(mirror: Option<f64>) -> Tf {
    match mirror {
        Some(m) => [-CM, 0.0, 0.0, CM, m * CM, 0.0],
        None => [CM, 0.0, 0.0, CM, 0.0, 0.0],
    }
}
pub fn get_transform() -> Tf { with(|s| s.t) }
pub fn push() { with(|s| { let t = s.t; s.stack.push(t) }) }
pub fn pop() { with(|s| if let Some(t) = s.stack.pop() { s.t = t }) }
pub fn translate(x: f64, y: f64) {
    with(|s| {
        let t = s.t;
        s.t = [t[0], t[1], t[2], t[3], t[0] * x + t[2] * y + t[4], t[1] * x + t[3] * y + t[5]];
    })
}
pub fn rotate(deg: f64) {
    with(|s| {
        let t = s.t;
        let (c, sn) = ((deg * RAD).cos(), (deg * RAD).sin());
        s.t = [t[0] * c + t[2] * sn, t[1] * c + t[3] * sn, -t[0] * sn + t[2] * c, -t[1] * sn + t[3] * c, t[4], t[5]];
    })
}
pub fn scale(sx: f64, sy: Option<f64>) {
    let sy = sy.unwrap_or(sx);
    with(|s| {
        let t = s.t;
        s.t = [t[0] * sx, t[1] * sx, t[2] * sy, t[3] * sy, t[4], t[5]];
    })
}
/// size of one local unit on the page, in cm (for level-of-detail decisions)
pub fn unit() -> f64 { with(|s| (s.t[0] * s.t[3] - s.t[1] * s.t[2]).abs().sqrt() / CM) }
/// local point -> page cm
pub fn to_page(x: f64, y: f64) -> (f64, f64) {
    with(|s| {
        let t = s.t;
        ((t[0] * x + t[2] * y + t[4]) / CM, (t[1] * x + t[3] * y + t[5]) / CM)
    })
}

// ---------------------------------------------------------------- output
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Kind { Stem, #[default] Bloom, Overlay }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase { Stems, Ghost, Branch, Fill, Accent, Tail, Marks, Fall }

/// The animation event a tag refers to: the table grow.lua builds (kind, phase, parent,
/// the fields its timing reads) and a theme may build with just `t0` and `dur`.
/// Events are shared and mutated after creation, as the Lua tables are. `t0` is NaN
/// until the schedule has placed the event (Lua's nil).
#[derive(Default)]
pub struct EventData {
    pub t0: f64,
    pub dur: f64,
    pub kind: Kind,
    pub phase: Option<Phase>,
    pub parent: Option<Event>,
    /// the phase of the root of the parent chain (the Lua's `e.root.phase`)
    pub root_phase: Option<Phase>,
    pub i: usize,
    pub at: Option<f64>,
    pub size: Option<f64>,
    pub len: Option<f64>,
    pub off: Option<f64>,
    pub span: Option<f64>,
    pub till_end: bool,
    pub lead: f64,
}
pub type Event = Rc<RefCell<EventData>>;

/// What `ink.tag` is given: the event, the fraction of it at which the piece appears
/// (`from`, up to `to` if any) and whether it belongs to the overlay.
#[derive(Clone)]
pub struct Tag {
    pub event: Event,
    pub from: f64,
    pub to: Option<f64>,
    pub overlay: bool,
}

pub fn layer(n: i32) -> i32 { with(|s| std::mem::replace(&mut s.cur, n)) }
pub fn tag(t: Option<Tag>) -> Option<Tag> { with(|s| std::mem::replace(&mut s.tag, t)) }
pub fn record(on: bool) { with(|s| { s.recording = on; s.tag = None }) }

fn gs(op: f64, mult: bool) -> Option<String> {
    let op = (op * 100.0 + 0.5).floor() as i64;
    if op >= 100 && !mult { return None; }
    Some(format!("flo{}{}", if mult { "m" } else { "a" }, op))
}

// 0.1 bp (0.035 mm) is finer than any printer resolves
fn pt_into(o: &mut String, t: &Tf, x: f64, y: f64) {
    fixed(o, t[0] * x + t[2] * y + t[4], 1);
    o.push(' ');
    fixed(o, t[1] * x + t[3] * y + t[5], 1);
}

fn sep(p: &mut String) { if !p.is_empty() { p.push(' ') } }

fn op_pt(p: &mut String, t: &Tf, x: f64, y: f64, op: &str) {
    sep(p);
    pt_into(p, t, x, y);
    p.push(' ');
    p.push_str(op);
}

fn add_path(s: &str) { with(|st| { sep(&mut st.path); st.path.push_str(s) }) }

pub fn moveto(x: f64, y: f64) { with(|s| op_pt(&mut s.path, &s.t, x, y, "m")) }
pub fn lineto(x: f64, y: f64) { with(|s| op_pt(&mut s.path, &s.t, x, y, "l")) }
fn curve_pts(p: &mut String, t: &Tf, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) {
    sep(p);
    pt_into(p, t, x1, y1);
    p.push(' ');
    pt_into(p, t, x2, y2);
    p.push(' ');
    pt_into(p, t, x3, y3);
    p.push_str(" c");
}
pub fn curveto(x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) {
    with(|s| curve_pts(&mut s.path, &s.t, x1, y1, x2, y2, x3, y3))
}
pub fn close() { add_path("h") }
pub fn poly(pts: &[Pt], closed: bool) {
    with(|s| {
        op_pt(&mut s.path, &s.t, pts[0][0], pts[0][1], "m");
        for p in &pts[1..] { op_pt(&mut s.path, &s.t, p[0], p[1], "l") }
        if closed { s.path.push_str(" h") }
    })
}
pub fn smooth(pts: &[Pt], closed: bool, tension: Option<f64>) {
    let n = pts.len();
    if n < 3 { return poly(pts, closed); }
    let k = tension.unwrap_or(1.0) / 6.0;
    let at = |i: i64| -> Pt {
        if closed { pts[(i - 1).rem_euclid(n as i64) as usize] }
        else { pts[(i.min(n as i64).max(1) - 1) as usize] }
    };
    with(|s| {
        op_pt(&mut s.path, &s.t, pts[0][0], pts[0][1], "m");
        let last = if closed { n } else { n - 1 } as i64;
        for i in 1..=last {
            let (p0, p1, p2, p3) = (at(i - 1), at(i), at(i + 1), at(i + 2));
            curve_pts(&mut s.path, &s.t,
                p1[0] + (p2[0] - p0[0]) * k, p1[1] + (p2[1] - p0[1]) * k,
                p2[0] - (p3[0] - p1[0]) * k, p2[1] - (p3[1] - p1[1]) * k,
                p2[0], p2[1]);
        }
        if closed { s.path.push_str(" h") }
    })
}
pub fn circle(x: f64, y: f64, r: f64) {
    let k = 0.5523 * r;
    with(|s| {
        let t = s.t;
        let p = &mut s.path;
        op_pt(p, &t, x + r, y, "m");
        curve_pts(p, &t, x + r, y + k, x + k, y + r, x, y + r);
        curve_pts(p, &t, x - k, y + r, x - r, y + k, x - r, y);
        curve_pts(p, &t, x - r, y - k, x - k, y - r, x, y - r);
        curve_pts(p, &t, x + k, y - r, x + r, y - k, x + r, y);
        p.push_str(" h");
    })
}

fn rgb_into(o: &mut String, c: Color, stroke: bool) {
    for (i, v) in c.iter().enumerate() {
        if i > 0 { o.push(' ') }
        fixed(o, *v, 3);
    }
    o.push_str(if stroke { " RG" } else { " rg" });
}

fn gs_prefix(o: &mut String, g: &Option<String>) {
    if let Some(g) = g { o.push_str("q /"); o.push_str(g); o.push_str(" gs ") }
}

/// paint the current path. opacity < 1 or mult=true wraps it in its own graphics state
pub fn fill(c: Color, op: Option<f64>, mult: bool) {
    let g = gs(op.unwrap_or(1.0), mult);
    with(|s| {
        let p = s.take();
        let mut o = String::with_capacity(p.len() + 48);
        gs_prefix(&mut o, &g);
        rgb_into(&mut o, c, false);
        o.push(' ');
        o.push_str(&p);
        o.push_str(if g.is_some() { " f Q" } else { " f" });
        s.put(o)
    })
}
/// dash: a PDF dash array as the Lua takes it, a string such as "[2 2] 0"
pub fn stroke(c: Color, w: f64, op: Option<f64>, mult: bool, dash: Option<&str>) {
    let g = gs(op.unwrap_or(1.0), mult);
    with(|s| {
        let p = s.take();
        let mut o = String::with_capacity(p.len() + 64);
        if g.is_some() { gs_prefix(&mut o, &g) } else { o.push_str("q ") }
        rgb_into(&mut o, c, true);
        o.push(' ');
        fixed(&mut o, w, 2);
        o.push_str(" w ");
        if let Some(d) = dash { o.push_str(d); o.push_str(" d ") }
        o.push_str(&p);
        o.push_str(" S Q");
        s.put(o)
    })
}
/// clip to the current path until the matching unclip()
pub fn clip() { with(|s| { let p = s.take(); s.put(format!("q {} W n", p)) }) }
pub fn unclip() { with(|s| s.put("Q".into())) }
pub fn discard() { with(|s| s.path.clear()) }
/// paint the whole of the current clip
pub fn flood(c: Color, x0: f64, y0: f64, x1: f64, y1: f64) {
    poly(&[[x0, y0], [x1, y0], [x1, y1], [x0, y1]], true);
    fill(c, None, false);
}

// write all layers (in order) as one literal; returns its length
fn literal(parts: Vec<String>) -> usize {
    if parts.is_empty() { return 0; }
    let s = format!("q 1 J 1 j\n{}\nQ", parts.join("\n"));
    let n = s.len();
    SINK.with(|k| k.borrow_mut().push(s));
    n
}

pub fn flush() -> usize {
    let layers = with(|s| {
        s.tags.clear();
        std::mem::take(&mut s.layers)
    });
    literal(layers.values().map(|l| l.join("\n")).collect())
}

/// What `take_recording` hands over instead of writing: the pieces with their tags.
pub struct Recording {
    pub keys: Vec<i32>,
    pub layers: BTreeMap<i32, Vec<String>>,
    pub tags: BTreeMap<i32, Vec<Option<Tag>>>,
}

pub fn take_recording() -> Recording {
    with(|s| {
        let layers = std::mem::take(&mut s.layers);
        let tags = std::mem::take(&mut s.tags);
        s.recording = false;
        s.tag = None;
        Recording { keys: layers.keys().copied().collect(), layers, tags }
    })
}

/// write one frame of a recording: the untagged pieces and those keep(tag) accepts
pub fn emit(rec: &Recording, keep: &dyn Fn(&Tag) -> bool) -> usize {
    let mut parts = vec![];
    for k in &rec.keys {
        let (l, t) = (&rec.layers[k], &rec.tags[k]);
        let out: Vec<&str> = (0..l.len())
            .filter(|&i| match &t[i] { None => true, Some(tag) => keep(tag) })
            .map(|i| l[i].as_str())
            .collect();
        if !out.is_empty() { parts.push(out.join("\n")) }
    }
    literal(parts)
}

// ---------------------------------------------------------------- geometry
// polyline helpers: points are [x,y]; `spine` functions return (x,y) for v in [0,1]

/// outward normals of a polyline
pub fn normals(pts: &[Pt]) -> Vec<Pt> {
    let n = pts.len();
    (1..=n).map(|i| {
        let (a, b) = (pts[i.saturating_sub(1).max(1) - 1], pts[(i + 1).min(n) - 1]);
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let mut l = (dx * dx + dy * dy).sqrt();
        if l < 1e-9 { l = 1.0 }
        [-dy / l, dx / l]
    }).collect()
}

/// cumulative length of a polyline, and the total
pub fn arclen(pts: &[Pt]) -> (Vec<f64>, f64) {
    let (mut s, mut acc) = (vec![0.0], 0.0);
    for i in 1..pts.len() {
        let (dx, dy) = (pts[i][0] - pts[i - 1][0], pts[i][1] - pts[i - 1][1]);
        acc = acc + (dx * dx + dy * dy).sqrt();
        s.push(acc);
    }
    (s, acc)
}

/// a ribbon along a centerline: width(u) is the full width at fraction u of the length.
/// Returns the closed outline (left side forward, right side back).
pub fn ribbon(pts: &[Pt], width: &dyn Fn(f64) -> f64) -> Vec<Pt> {
    let nrm = normals(pts);
    let (s, l) = arclen(pts);
    let (mut left, mut right) = (vec![], vec![]);
    for (i, p) in pts.iter().enumerate() {
        let w = width(if l > 0.0 { s[i] / l } else { 0.0 }) / 2.0;
        left.push([p[0] + nrm[i][0] * w, p[1] + nrm[i][1] * w]);
        right.push([p[0] - nrm[i][0] * w, p[1] - nrm[i][1] * w]);
    }
    let mut out = left;
    out.extend(right.into_iter().rev());
    out
}

/// a tapered stroke along one cubic Bézier, as a closed outline of two Béziers:
/// the control points are pushed out along the curve's normals by half the
/// width at their parameter (w0 at the start, w1 at the end).
#[allow(clippy::too_many_arguments)]
pub fn taper(x0: f64, y0: f64, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, w0: f64, w1: f64) {
    let nrm = |dx: f64, dy: f64| {
        let l = (dx * dx + dy * dy).sqrt();
        if l < 1e-9 { (0.0, 0.0) } else { (-dy / l, dx / l) }
    };
    let (n0x, n0y) = nrm(x1 - x0, y1 - y0);
    let (n1x, n1y) = nrm(x2 - x0, y2 - y0);
    let (n2x, n2y) = nrm(x3 - x1, y3 - y1);
    let (n3x, n3y) = nrm(x3 - x2, y3 - y2);
    let (h0, h1, h2, h3) = (w0 / 2.0, (2.0 * w0 + w1) / 6.0, (w0 + 2.0 * w1) / 6.0, w1 / 2.0);
    with(|s| {
        let t = s.t;
        let p = &mut s.path;
        op_pt(p, &t, x0 + n0x * h0, y0 + n0y * h0, "m");
        curve_pts(p, &t, x1 + n1x * h1, y1 + n1y * h1, x2 + n2x * h2, y2 + n2y * h2, x3 + n3x * h3, y3 + n3y * h3);
        op_pt(p, &t, x3 - n3x * h3, y3 - n3y * h3, "l");
        curve_pts(p, &t, x2 - n2x * h2, y2 - n2y * h2, x1 - n1x * h1, y1 - n1y * h1, x0 - n0x * h0, y0 - n0y * h0);
        p.push_str(" h");
    })
}

/// Catmull-Rom samples through control points, `per` per segment
pub fn catmull(pts: &[Pt], per: usize) -> Vec<Pt> {
    let n = pts.len();
    let mut out = vec![];
    for i in 1..n {
        let (p0, p1, p2, p3) = (pts[(i - 1).max(1) - 1], pts[i - 1], pts[i], pts[(i + 2).min(n) - 1]);
        for k in 0..per {
            let t = k as f64 / per as f64;
            let (t2, t3) = (t * t, t * t * t);
            let c = |j: usize| 0.5 * (2.0 * p1[j] + (-p0[j] + p2[j]) * t
                + (2.0 * p0[j] - 5.0 * p1[j] + 4.0 * p2[j] - p3[j]) * t2
                + (-p0[j] + 3.0 * p1[j] - 3.0 * p2[j] + p3[j]) * t3);
            out.push([c(0), c(1)]);
        }
    }
    out.push([pts[n - 1][0], pts[n - 1][1]]);
    out
}

/// sample a curve function f(v) -> (x,y) at n+1 points
pub fn sample(f: &dyn Fn(f64) -> (f64, f64), n: usize, v0: Option<f64>, v1: Option<f64>) -> Vec<Pt> {
    let (v0, v1) = (v0.unwrap_or(0.0), v1.unwrap_or(1.0));
    (0..=n).map(|i| {
        let (x, y) = f(v0 + (v1 - v0) * i as f64 / n as f64);
        [x, y]
    }).collect()
}

/// cubic Bézier evaluator, a function of v
pub fn bezier3(x0: f64, y0: f64, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> impl Fn(f64) -> (f64, f64) {
    move |v| {
        let u = 1.0 - v;
        let (a, b, c, d) = (u * u * u, 3.0 * u * u * v, 3.0 * u * v * v, v * v * v);
        (a * x0 + b * x1 + c * x2 + d * x3, a * y0 + b * y1 + c * y2 + d * y3)
    }
}

/// a blade: closed outline around a spine with half-widths wl(v), wr(v) on each side
/// and an optional edge perturbation e(v, side) added along the normal.
/// Returns the outline, the spine samples and their normals.
pub fn blade(spine: &dyn Fn(f64) -> (f64, f64), n: usize, wl: &dyn Fn(f64) -> f64, wr: &dyn Fn(f64) -> f64,
             edge: Option<&dyn Fn(f64, f64) -> f64>) -> (Vec<Pt>, Vec<Pt>, Vec<Pt>) {
    let c = sample(spine, n, None, None);
    let nrm = normals(&c);
    let (mut left, mut right) = (vec![], vec![]);
    for i in 1..=n + 1 {
        let v = (i - 1) as f64 / n as f64;
        let (mut el, mut er) = (0.0, 0.0);
        if let Some(e) = edge { el = e(v, 1.0); er = e(v, -1.0) }
        let (mut a, mut b) = (wl(v) + el, wr(v) + er);
        if a < 0.0 { a = 0.0 }
        if b < 0.0 { b = 0.0 }
        let (ci, ni) = (c[i - 1], nrm[i - 1]);
        left.push([ci[0] + ni[0] * a, ci[1] + ni[1] * a]);
        right.push([ci[0] - ni[0] * b, ci[1] - ni[1] * b]);
    }
    let mut out: Vec<Pt> = left;
    for i in (2..=n).rev() { out.push(right[i - 1]) }
    (out, c, nrm)
}

/// an irregular blob: a circle of radius r whose radius wobbles by `rough`
pub fn blob(rng: &Rng, x: f64, y: f64, rx: f64, ry: f64, rough: f64, n: Option<i32>) -> Vec<Pt> {
    let n = n.unwrap_or(9);
    let ph = rng.range(0.0, 2.0 * PI);
    let ph2 = rng.range(0.0, 2.0 * PI);
    let rot = rng.range(0.0, 2.0 * PI);
    (0..n).map(|i| {
        let a = 2.0 * PI * i as f64 / n as f64;
        let k = 1.0 + rough * (0.6 * (2.0 * a + ph).sin() + 0.4 * (3.0 * a + ph2).sin()) + rough * 0.5 * rng.sym(1.0);
        let (px, py) = (rx * k * a.cos(), ry * k * a.sin());
        [x + px * rot.cos() - py * rot.sin(), y + px * rot.sin() + py * rot.cos()]
    }).collect()
}
