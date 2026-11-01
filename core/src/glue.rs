// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! florilegium.lua: the TeX-facing side of florilegium.sty without TeX: page geometry,
//! colors, seeds, the dispatch of primitives, ornaments, initials, the head rule, captions,
//! the growth animation's frames and the grown and authored pages.
//!
//! What the Lua keeps in module variables (the page rng, the current colors) lives in
//! thread-local state here; `reset` puts it back. The theme in effect (`M.theme`) is an
//! argument instead, and so is the ornament count (`nornament`). Every entry point that
//! can fail leaves the ink state clean (`ink::reset`) when it does.
use crate::flora::{self, Opts};
use crate::grow::{self, Caption, Mark, G};
use crate::ink::{self, Rng, CM};
use crate::initial::{self, Shapes};
use crate::{authored, figures, geometric, svg, tex};
use std::cell::{Cell, RefCell};

/// cm per scaled point
pub const SP_CM: f64 = 2.54 / (72.27 * 65536.0);
const INCH: f64 = 2.54;

// ---------------------------------------------------------------- geometry
/// the TeX lengths the page layout is made of, in sp
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Dimens {
    pub paperwidth: i64, pub paperheight: i64, pub textwidth: i64, pub textheight: i64,
    pub oddsidemargin: i64, pub evensidemargin: i64, pub topmargin: i64,
    pub headheight: i64, pub headsep: i64, pub footskip: i64, pub hoffset: i64,
}

impl Dimens {
    /// svg.lua's formulas: the inner margin on odd pages is `left`; on even pages of a
    /// two-sided document the text moves over by the difference
    pub fn from_margins(pw: i64, ph: i64, left: i64, right: i64, top: i64, bottom: i64,
                        headheight: i64, headsep: i64, footskip: i64) -> Result<Dimens, String> {
        let (tw, th) = (pw - left - right, ph - top - bottom);
        if tw <= 0 || th <= 0 { return Err("florilegium: the margins leave no room for the text".into()) }
        let inch = svg::sp("1in")?;
        Ok(Dimens {
            paperwidth: pw, paperheight: ph, textwidth: tw, textheight: th,
            oddsidemargin: left - inch, evensidemargin: pw - tw - left - inch,
            topmargin: top - inch - headheight - headsep, headheight, headsep, footskip, hoffset: 0,
        })
    }
}

/// the page layout, in cm from the lower-left corner of the paper (`M.geometry`)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Geometry {
    pub pw: f64, pub ph: f64,
    pub textleft: f64, pub textright: f64,
    pub ytop: f64, pub ybot: f64,
    pub colright: f64, pub headrule: f64, pub footy: f64,
    pub mirror: bool,
}

/// The garden grows in the wider side margin: when that is the right one (the outer margin
/// of a right-hand page in a two-sided document), the layout is made for the mirrored page
/// and drawn reflected.
pub fn geometry(d: &Dimens, page: i64, twoside: bool) -> Geometry {
    let cm = |v: i64| v as f64 * SP_CM;
    let even = twoside && page.rem_euclid(2) == 0;
    let (pw, ph) = (cm(d.paperwidth), cm(d.paperheight));
    let mut textleft = INCH + d.hoffset as f64 * SP_CM + cm(if even { d.evensidemargin } else { d.oddsidemargin });
    let mut textright = textleft + cm(d.textwidth);
    let mut mirror = false;
    if pw - textright > textleft + 0.05 {
        mirror = true;
        let (l, r) = (pw - textright, pw - textleft);
        textleft = l;
        textright = r;
    }
    let ytop = ph - INCH - cm(d.topmargin) - cm(d.headheight) - cm(d.headsep);
    let ybot = ytop - cm(d.textheight);
    Geometry {
        pw, ph, textleft, textright, ytop, ybot, mirror,
        colright: textleft - 0.4,
        headrule: ytop + cm(d.headsep) - 0.1,
        footy: ybot - cm(d.footskip),
    }
}

// ---------------------------------------------------------------- state
const DEFAULT_COLORS: [f64; 6] = [0x24 as f64 / 255.0, 0x56 as f64 / 255.0, 0xC4 as f64 / 255.0, 0.71, 0.40, 0.18];

thread_local! {
    static CURRENT: Cell<[f64; 6]> = const { Cell::new(DEFAULT_COLORS) };
    static PAGE_RNG: RefCell<Rng> = RefCell::new(Rng::new(1.0));
}

/// A fresh Lua process: the default colors and the page rng seeded with 1.
pub fn reset() {
    CURRENT.with(|c| c.set(DEFAULT_COLORS));
    PAGE_RNG.with(|r| *r.borrow_mut() = Rng::new(1.0));
}

fn page_rng() -> Rng { PAGE_RNG.with(|r| r.borrow().clone()) }

/// `ink` sets the hue of the whole ramp, `accent` the copper
pub fn colors(ir: f64, ig: f64, ib: f64, ar: f64, ag: f64, ab: f64) {
    CURRENT.with(|c| c.set([ir, ig, ib, ar, ag, ab]));
    ink::set_ink(ir, ig, ib);
    ink::set_accent(ar, ag, ab);
}

/// one rng per page; primitives drawn from TeX fork it in call order. Lua-faithful state for
/// the oracle and the CLI: NOT part of the plugin API (the plugin passes explicit seeds).
pub fn seed(page: i64, fixed: Option<i64>) -> i64 {
    let s = match fixed {
        Some(f) => f.wrapping_mul(7919).wrapping_add(page.wrapping_mul(131)),
        None => page.wrapping_mul(97).wrapping_add(1030),
    };
    PAGE_RNG.with(|r| *r.borrow_mut() = Rng::new(s as f64));
    s
}

/// every entry point starts from a fresh ink state (pale, lift, colors, transform, sink) and
/// the default current colors, so no call depends on an earlier one; on error nothing is left
/// half drawn
fn entry<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    ink::reset();
    CURRENT.with(|c| c.set(DEFAULT_COLORS));
    let r = f();
    if r.is_err() {
        ink::reset();
        reset();
    }
    r
}

fn hex(h: &str) -> Result<(f64, f64, f64), String> {
    if h.len() != 6 || !h.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("florilegium: not a color (RRGGBB): '{h}'"));
    }
    let c = |a: usize| u8::from_str_radix(&h[a..a + 2], 16).unwrap() as f64 / 255.0;
    Ok((c(0), c(2), c(4)))
}

/// `ink` and `accent` (RRGGBB, "" leaves that color as it is) for the duration of `f`
fn with_colors<T>(inkhex: &str, acchex: &str, f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let c = CURRENT.with(|c| c.get());
    let changed = !inkhex.is_empty() || !acchex.is_empty();
    if changed {
        let i = if inkhex.is_empty() { (c[0], c[1], c[2]) } else { hex(inkhex)? };
        let a = if acchex.is_empty() { (c[3], c[4], c[5]) } else { hex(acchex)? };
        colors(i.0, i.1, i.2, a.0, a.1, a.2);
    }
    let r = f();
    if changed { colors(c[0], c[1], c[2], c[3], c[4], c[5]) }
    r
}

// ---------------------------------------------------------------- draw
/// \florilegiumdraw{name}{s, opts}: one primitive at the origin of the current scope (here
/// the origin of the page, in cm); the names of the theme's primitives work too. Returns the
/// SVG of it. `seed` seeds the rng the primitive's own is forked from (florilegium.lua's
/// `page_rng.fork()`); "" colors are ink's defaults.
pub fn draw(name: &str, size: f64, o: &Opts, inkhex: &str, acchex: &str, theme: Option<&str>, seed: i64) -> Result<String, String> {
    entry(|| with_colors(inkhex, acchex, || {
        ink::set_transform([CM, 0.0, 0.0, CM, 0.0, 0.0]);
        let rng = Rng::new(seed as f64).fork();
        grow::draw_named(theme, name, &rng, size, o)?;
        ink::flush();
        svg::standalone(&ink::take_output())
    }))
}

/// \florilegiumdraw{ghost}{"flower", {s=1.4}}: any primitive drawn pale (`o.lift`)
pub fn draw_ghost(inner: &str, size: f64, o: &Opts, inkhex: &str, acchex: &str, theme: Option<&str>, seed: i64) -> Result<String, String> {
    entry(|| with_colors(inkhex, acchex, || {
        ink::set_transform([CM, 0.0, 0.0, CM, 0.0, 0.0]);
        let rng = Rng::new(seed as f64).fork();
        let mut res = Ok(());
        flora::ghost(o.lift, || res = grow::draw_named(theme, inner, &rng, size, o));
        res?;
        ink::flush();
        svg::standalone(&ink::take_output())
    }))
}

// ---------------------------------------------------------------- ornaments in the text
/// where each primitive sits in the box of side `size`: its origin (x, y as fractions of the
/// side), its scale, its angle, and options for it
fn placement(name: &str) -> (f64, f64, f64, f64, Opts) {
    let o = Opts::default();
    match name {
        "leaf" => (0.2, 0.1, 1.02, -40.0, o),
        "leaflet" => (0.2, 0.1, 1.0, -40.0, o),
        "petal" => (0.3, 0.02, 1.0, -30.0, o),
        "bud" => (0.35, 0.0, 0.95, -20.0, o),
        "bell" => (0.5, 0.97, 0.95, 0.0, o),
        "sprig" => (0.4, 0.0, 1.0, -10.0, o),
        "cyme" => (0.35, 0.0, 0.95, -10.0, Opts { side: Some(-1.0), ..o }),
        "umbel" => (0.5, 0.05, 0.8, 0.0, o),
        "tendril" => (0.95, 0.5, 0.45, 0.0, o),
        "flower" => (0.5, 0.5, 0.47, 0.0, o),
        "blossom" => (0.5, 0.5, 0.48, 0.0, o),
        "floret" => (0.5, 0.5, 0.46, 0.0, o),
        "lily" => (0.5, 0.45, 0.6, 0.0, o),
        "seedhead" => (0.5, 0.42, 0.42, 0.0, Opts { drift: Some(0.0), ..o }),
        "butterfly" => (0.5, 0.5, 0.98, 0.0, o),
        "bee" => (0.5, 0.5, 0.8, -30.0, Opts { trail: Some(false), ..o }),
        "snail" => (0.5, 0.35, 0.55, -90.0, o),
        "bird" => (0.5, 0.02, 0.95, 0.0, o),
        "sparkle" => (0.5, 0.5, 0.3, 0.0, o),
        "rosette" => (0.5, 0.5, 0.38, 0.0, o),
        "hedera" => (0.45, 0.62, 0.36, 0.0, o),
        _ => (0.5, 0.5, 0.46, 0.0, o),
    }
}

/// \florilegiumornament[size, ink, accent, angle, seed]{name}: one primitive in a box of side
/// `size` (sp) on the baseline, like a letter (its depth 0.15 of the size), as an SVG with its
/// origin at the baseline. The k-th ornament of the document (`count`, from 1) always gets the
/// same shape, unless a seed is given.
pub fn ornament(name: &str, size_sp: i64, inkhex: &str, acchex: &str, angle: f64, seed: Option<i64>, count: i64,
                theme: Option<&str>) -> Result<String, String> {
    entry(|| {
        let s = match seed {
            Some(sd) => sd.wrapping_mul(7919).wrapping_add(17),
            None => count.wrapping_mul(7919).wrapping_add(4243),
        };
        with_colors(inkhex, acchex, || {
            let big_s = size_sp as f64 * SP_CM;
            let p = placement(name);
            ink::set_transform([CM, 0.0, 0.0, CM, 0.0, 0.0]);
            ink::translate(big_s * p.0, big_s * (p.1 - 0.15));
            ink::rotate(p.3 + angle);
            grow::draw_named(theme, name, &Rng::new(s as f64), big_s * p.2, &p.4)?;
            ink::flush();
            svg::standalone(&ink::take_output())
        })
    })
}

// ---------------------------------------------------------------- the head
/// the header: a copper ivy leaf (or the theme's fleuron) whose stem becomes the head rule
pub fn head(g: &Geometry, theme: Option<&str>) -> Result<(), String> {
    grow::theme(theme)?;
    ink::set_transform(ink::page_transform(if g.mirror { Some(g.pw) } else { None }));
    ink::push();
    ink::translate(g.colright - 0.55, g.headrule + 0.02);
    let rng = page_rng().fork();
    let (mut x, y) = match theme {
        None | Some("") | Some("flora") => flora::hedera(&rng, 0.36, &Opts::default()),
        Some("geometric") => geometric::hedera(&rng, 0.36, &Opts::default()),
        Some("math") => figures::compass(&rng, 0.36, None),
        Some(t) => return Err(format!("florilegium: no theme '{t}'")),
    };
    if g.mirror { x = g.pw - x }
    ink::pop();
    let e = g.textright;
    ink::moveto(x, y);
    ink::curveto(x + 1.0, y, x + 1.5, g.headrule, x + 2.5, g.headrule);
    ink::lineto(e, g.headrule);
    ink::stroke(ink::accent(0.0), 0.4, None, false, None);
    ink::circle(e, g.headrule, 0.03);
    ink::fill(ink::accent(0.0), None, false);
    ink::flush();
    Ok(())
}

// ---------------------------------------------------------------- initials
/// \florilegiuminitial: a tile of side `side_sp` whose top is the cap height above the current
/// baseline and whose left edge is `hang_sp` to the left of the current point; returns its SVG.
/// `seed` seeds the page rng the tile's rng is forked from.
pub fn initial(letter: &str, side_sp: i64, hang_sp: i64, cap_sp: i64, shapes: &Shapes, seed: i64) -> Result<String, String> {
    entry(|| {
        let bp = 72.0 / 72.27 / 65536.0;
        ink::set_transform([CM, 0.0, 0.0, CM, -(hang_sp as f64) * bp, (cap_sp - side_sp) as f64 * bp]);
        let rng = Rng::new(seed as f64).fork();
        initial::draw(&rng, side_sp as f64 * SP_CM, letter, shapes).map_err(|e| format!("florilegium: {e}"))?;
        ink::flush();
        svg::standalone(&ink::take_output())
    })
}

/// a garden's age over a document: 0 spring (buds), 0.5 summer, 1 autumn; from the page and
/// the page count of the last run
pub fn auto_age(page: i64, last: i64) -> f64 {
    if last > 1 { f64::min(1.0, (page - 1) as f64 / (last - 1) as f64) } else { 0.5 }
}

// ---------------------------------------------------------------- the page
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode { Authored, Grown }

/// frame k of n of a grown garden's growth (`\florilegiumframe[explain]{k}{n}`)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame { pub k: i64, pub n: i64, pub explain: bool }

#[derive(Clone, Debug)]
pub struct PageArgs {
    pub page: i64,
    pub twoside: bool,
    pub dimens: Dimens,
    pub seed: Option<i64>,
    pub mode: Mode,
    /// the theme in effect on this page (a theme mark changes it from the page it is on)
    pub theme: Option<String>,
    /// the garden's age: 0 spring, 0.5 summer, 1 autumn; None leaves it as it always was
    pub season: Option<f64>,
    /// headings and blooms on the page, y in page cm; theme marks are resolved by the caller
    pub marks: Vec<Mark>,
    pub density: f64,
    pub intensity: f64,
    pub visitors: bool,
    pub ink: String,
    pub accent: String,
    /// the page holds the end of the text: the garden spills below this y (cm)
    pub spill: Option<f64>,
    pub frame: Option<Frame>,
    pub nohead: bool,
    pub id: Option<String>,
    pub background: Option<String>,
}

impl Default for PageArgs {
    fn default() -> Self {
        PageArgs {
            page: 1, twoside: false, dimens: Dimens::default(), seed: None, mode: Mode::Grown, theme: None, season: None,
            marks: vec![], density: 1.0, intensity: 1.0, visitors: true, ink: "2456C4".into(), accent: "B4662F".into(),
            spill: None, frame: None, nohead: false, id: None, background: None,
        }
    }
}

/// a theme's text in the margin, placed as florilegium.lua's `captions()` mirrors it for TikZ
#[derive(Clone, Debug, PartialEq)]
pub struct PlacedCaption {
    pub x_cm: f64, pub y_cm: f64,
    pub anchor: String,
    pub size_pt: f64,
    /// degrees, counterclockwise
    pub rotate: f64,
    pub color: [u8; 3],
    pub opacity: f64,
    /// the TeX of the Lua
    pub tex: String,
    /// the same as Typst markup
    pub markup: String,
}

#[derive(Debug)]
pub struct Garden {
    /// the PDF literals of the page, in order; the head rule is the last
    pub streams: Vec<String>,
    pub captions: Vec<PlacedCaption>,
    /// what grew, one fact per line (empty for the authored page and the math theme)
    pub stats: String,
    pub geometry: Geometry,
}

fn pale(intensity: f64) -> f64 { 0.85 * (1.0 - f64::max(0.0, f64::min(1.0, intensity))) }

fn base_g(geo: &Geometry, seed: f64) -> G {
    G {
        pw: geo.pw, ph: geo.ph, textleft: geo.textleft, textright: geo.textright, ytop: geo.ytop, ybot: geo.ybot,
        colright: geo.colright, headrule: geo.headrule, footy: geo.footy, mirror: geo.mirror, seed, ..Default::default()
    }
}

/// the growth animation: the garden is recorded and the part grown so far is written; a
/// caption shows once its event is over
fn grown_frame(g: &mut G, f: &Frame) -> Result<Vec<Caption>, String> {
    g.anim = true;
    let r = grow::page(g)?;
    let rec = r.rec.ok_or("florilegium: no recording")?;
    // the last frame is the page: every event finished
    let t = if f.k >= f.n { f64::INFINITY } else { r.total * (f.k - 1) as f64 / i64::max(1, f.n - 1) as f64 };
    ink::emit(&rec, &|tag| {
        if tag.overlay && !f.explain { return false }
        let e = tag.event.borrow();
        let p = (t - e.t0) / e.dur;
        p >= tag.from && tag.to.map_or(true, |to| p < to)
    });
    let mut shown = vec![];
    for cap in r.captions {
        let done = {
            let e = cap.event.as_ref().ok_or("florilegium: caption without event")?.borrow();
            t >= e.t0 + e.dur
        };
        if done { shown.push(cap) }
    }
    Ok(shown)
}

fn stats_lines(st: &grow::Stats) -> String {
    let mut s = format!("motifs {} stems {} cells {}", st.motifs, st.stems, st.cells);
    for (k, v) in &st.counts { s.push_str(&format!("\ncount {k} {v}")) }
    s
}

/// The garden of a page as the TeX glue draws it: the literals ink wrote, the text the theme
/// sets in the margin and what grew.
pub fn garden(a: &PageArgs) -> Result<Garden, String> {
    entry(|| garden_inner(a))
}

fn garden_inner(a: &PageArgs) -> Result<Garden, String> {
    let i = svg::html(&a.ink)?;
    let c = svg::html(&a.accent)?;
    colors(i[0], i[1], i[2], c[0], c[1], c[2]);
    if let Some(s) = a.seed {
        if s < 1 { return Err("florilegium: a seed is a positive integer".into()) }
    }
    let seed = seed(a.page, a.seed);
    let geo = geometry(&a.dimens, a.page, a.twoside);
    let theme = a.theme.as_deref().filter(|t| !t.is_empty() && *t != "flora");
    let mut g = base_g(&geo, seed as f64);
    g.visitors = Some(a.visitors);
    g.theme = theme.map(str::to_string);
    let old = ink::set_pale(pale(a.intensity));
    let (caps, stats) = match a.mode {
        Mode::Authored => {
            let r = authored::page(&g)?;
            (r.captions, r.stats)
        }
        Mode::Grown => {
            g.marks = a.marks.clone();
            g.age = a.season;
            if let Some(y) = a.spill { g.spill = 1; g.spilly = y }
            if a.density < 1.0 { g.density = Some(f64::max(0.03, a.density)) }
            match &a.frame {
                Some(f) => (grown_frame(&mut g, f)?, None),
                None => {
                    let r = grow::page(&g)?;
                    (r.captions, r.stats)
                }
            }
        }
    };
    head(&geo, theme)?;
    ink::set_pale(old);

    // the captions as the TikZ nodes would set them
    let o = ink::set_pale(pale(a.intensity));
    let plain = ink::tone(0.12);
    ink::set_pale(o);
    let rgb255 = |c: [f64; 3]| [0, 1, 2].map(|k| (c[k] * 255.0 + 0.5).floor().clamp(0.0, 255.0) as u8);
    let mut captions = vec![];
    for cap in &caps {
        let mut anchor = cap.anchor.clone().unwrap_or_else(|| "north".into());
        if geo.mirror { anchor = anchor.replace("east", "\0").replace("west", "east").replace('\0', "west") }
        captions.push(PlacedCaption {
            x_cm: if geo.mirror { geo.pw - cap.x } else { cap.x },
            y_cm: cap.y,
            anchor,
            size_pt: match cap.scale { Some(s) => 10.0 * s, None => if cap.small { 5.5 } else { 6.5 } },
            rotate: match cap.rotate { Some(r) => if geo.mirror { -r } else { r }, None => 0.0 },
            color: rgb255(cap.color.unwrap_or(plain)),
            opacity: cap.opacity.unwrap_or(1.0),
            tex: cap.text.clone(),
            markup: tex::to_typst(&cap.text)?,
        });
    }
    ink::set_pale(0.0);
    Ok(Garden {
        streams: ink::take_output(),
        captions,
        stats: stats.as_ref().map(stats_lines).unwrap_or_default(),
        geometry: geo,
    })
}
