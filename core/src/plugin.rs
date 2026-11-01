// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! The wasm entry points: `page`, `ornament`, `draw` and `initial`, each taking one CBOR map
//! and returning one. The functions here are plain Rust over bytes (so tests run them
//! natively); the `#[wasm_func]` exports at the bottom exist only with the `plugin` feature.
//!
//! # Contract (what `src/core.typ` sends and reads)
//! Absent keys and CBOR `null` (Typst's `none`) mean the same. Numbers may be CBOR integers
//! or floats (all finite, |x| <= 1e12 unless said otherwise). Every failure is an `Err`
//! starting with "florilegium: ". Colors are hex `RRGGBB` (a leading `#` is accepted); `""`
//! or absent means the default. The seeds of `draw`, `ornament` and `initial` are any
//! integer (0 and negatives included; the arithmetic wraps). The document seed of `page`
//! is checked by the core as in the Lua: when given it must be >= 1.
//!
//! ## `page`
//! In: `page` int 1..=1e6; `twoside` bool (false); `dimens` map of integers in sp, all
//! required: paperwidth, paperheight, textwidth, textheight, oddsidemargin, evensidemargin,
//! topmargin, headheight, headsep, footskip, hoffset (|v| <= 2^30; paper sizes 1..=280e6 sp,
//! about 150 cm; text sizes > 0); `seed` int? (>= 1); `mode` "grown" (default) | "authored";
//! `theme` str?; `age` num? (0 spring, 0.5 summer, 1 autumn); `marks` array (<= 1000) of
//! `{kind: "heading"|"bloom", arg: int level | str motif ("" none), y: num}` with y in cm from
//! the page's lower-left corner; `density` num (1); `intensity` num (1); `visitors` bool
//! (true); `ink`, `accent` hex (defaults 2456C4, B4662F); `spilly` num? (cm, from the lower
//! left); `frame` `{k, n, explain}` with 1 <= k <= n <= 1e6; `nohead` bool; `background` hex?.
//! An unknown non-empty bloom motif is an error in grown mode.
//! Out: `{svg: bytes, captions: [{x, y, anchor, size, rotate, color, opacity, markup}],
//! mirror: bool, stats: str, geometry: {pw, ph, textleft, textright, ytop, ybot, colright,
//! headrule, footy}}`. `geometry` is in cm from the page's lower-left corner, after mirroring. The SVG is the whole page in bp (1 bp = 1 pt) with its
//! origin at the page's top-left. In captions, `x`, `y` are in pt from the page's top-left
//! corner, `size` in pt, `rotate` in degrees counterclockwise, `color` "#rrggbb",
//! `opacity` 0..1, `anchor` a TikZ anchor name, `markup` Typst markup.
//!
//! ## `ornament`
//! In: `name` str; `size` int sp (1..=1e8, about 53 cm); `angle` num degrees (0); `count`
//! int >= 1 (1; the k-th ornament of a document always looks the same); `seed` int?
//! (overrides `count`); `ink`, `accent`; `theme` str?.
//!
//! ## `draw`
//! In: `name` str; `size` num in cm (0 < size <= 50); `seed` int (required); `opts` map
//! (below); `ink`, `accent`; `theme` str?. With `opts.lift` present the primitive is drawn
//! pale through the ghost path. `opts` keys: tone w bend side open facing tilt drift lift
//! style k (|v| <= 1000); depth (0..=8), petals lobes streaks (0..=100); trail fork star
//! rays heart (bool); shape (str); eye color (hex); p v (arrays of <= 256 numbers,
//! |v| <= 1000; for `koch` and `hilbert` p[1] is 0..=7). Unknown keys are errors.
//!
//! ## `initial`
//! In: `letter` str; `side` int sp (1..=1e8); `font` bytes (an OpenType/TrueType file);
//! `seed` int; `hang` int sp (0), `cap` int sp (default `side`), both |v| <= 2^30. The tile's
//! left edge is `hang` left of the current point, its top `cap` above the baseline.
//!
//! ## Canvas (out of `ornament`, `draw`, `initial`)
//! `{svg: bytes, width, height, dx, dy}`, all lengths in pt (= bp). The canvas is the box of
//! every coordinate of every path (control points included), padded by
//! max(2 bp, widest stroke / 2 + 1 bp) and snapped outward to whole bp. `dx`, `dy` are the
//! position of the drawing's origin (the ornament's baseline-left, `draw`'s origin, the
//! initial's current point) from the canvas's top-left corner: place the image at
//! `(x - dx, y_top - dy)`. An empty drawing gives a 0 x 0 canvas, not an error.
use crate::flora::Opts;
use crate::glue::{self, Dimens, Frame, Mode, PageArgs};
use crate::grow::{Mark, MarkKind};
use crate::ink::Color;
use crate::{fmt, initial, svg};
use ciborium::Value;

type R<T> = Result<T, String>;

/// Caps that keep the work bounded (an `Err`, not a hang or a trap, beyond them); generous
/// enough that no sensible document reaches them.
const MAX_DIMEN_SP: i64 = 1 << 30; // TeX's own maximum, about 5.7 m
const MAX_PAPER_SP: i64 = 280_000_000; // about 150 cm; the free-space lattice grows with the paper
const MAX_BOX_SP: i64 = 100_000_000; // an ornament or initial: about 53 cm
const MAX_DRAW_CM: f64 = 50.0;
const MAX_MARKS: usize = 1000;
const MAX_OPT: f64 = 1000.0; // any other number of opts, p and v
const MAX_COUNT_OPT: f64 = 100.0; // petals, lobes, streaks
const MAX_DEPTH_OPT: f64 = 8.0; // depth: sprig recurses exponentially
const MAX_LEVEL_OPT: f64 = 7.0; // koch levels, hilbert order (p[1])

fn bad<T>(msg: impl AsRef<str>) -> R<T> { Err(format!("florilegium: {}", msg.as_ref())) }

// ---------------------------------------------------------------- decoding
struct Args<'a>(&'a [(Value, Value)]);

fn parse(bytes: &[u8]) -> R<Value> {
    ciborium::from_reader(bytes).or_else(|e| bad(format!("cannot decode the arguments: {e}")))
}

fn as_map<'a>(v: &'a Value, what: &str) -> R<Args<'a>> {
    match v {
        Value::Map(m) => Ok(Args(m)),
        _ => bad(format!("{what} must be a dictionary")),
    }
}

impl<'a> Args<'a> {
    fn get(&self, k: &str) -> Option<&'a Value> {
        self.0.iter().find(|(a, _)| a.as_text() == Some(k)).map(|(_, v)| v).filter(|v| !v.is_null())
    }
    fn req(&self, k: &str) -> R<&'a Value> { self.get(k).map_or_else(|| bad(format!("missing argument '{k}'")), Ok) }
    fn f64_in(&self, k: &str, lim: f64) -> R<Option<f64>> {
        match self.get(k) {
            None => Ok(None),
            Some(v) => Ok(Some(number(v, k, lim)?)),
        }
    }
    fn num(&self, k: &str) -> R<Option<f64>> { self.f64_in(k, 1e12) }
    fn num_req(&self, k: &str) -> R<f64> { self.num(k)?.map_or_else(|| bad(format!("missing argument '{k}'")), Ok) }
    fn int(&self, k: &str) -> R<Option<i64>> {
        match self.get(k) {
            None => Ok(None),
            Some(v) => Ok(Some(integer(v, k)?)),
        }
    }
    fn int_req(&self, k: &str) -> R<i64> { self.int(k)?.map_or_else(|| bad(format!("missing argument '{k}'")), Ok) }
    fn text(&self, k: &str) -> R<Option<&'a str>> {
        match self.get(k) {
            None => Ok(None),
            Some(Value::Text(s)) => Ok(Some(s.as_str())),
            Some(_) => bad(format!("'{k}' must be a string")),
        }
    }
    fn text_req(&self, k: &str) -> R<&'a str> { self.text(k)?.map_or_else(|| bad(format!("missing argument '{k}'")), Ok) }
    fn flag(&self, k: &str) -> R<Option<bool>> {
        match self.get(k) {
            None => Ok(None),
            Some(Value::Bool(b)) => Ok(Some(*b)),
            Some(_) => bad(format!("'{k}' must be a boolean")),
        }
    }
}

fn number(v: &Value, k: &str, lim: f64) -> R<f64> {
    let x = match v {
        Value::Float(x) => *x,
        Value::Integer(n) => i128::from(*n) as f64,
        _ => return bad(format!("'{k}' must be a number")),
    };
    if !x.is_finite() { return bad(format!("'{k}' must be a finite number")) }
    if x.abs() > lim { return bad(format!("'{k}' is out of range ({x})")) }
    Ok(x)
}

fn integer(v: &Value, k: &str) -> R<i64> {
    match v {
        Value::Integer(n) => i64::try_from(i128::from(*n)).or_else(|_| bad(format!("'{k}' is out of range"))),
        Value::Float(x) if x.is_finite() && x.fract() == 0.0 && x.abs() < 9e15 => Ok(*x as i64),
        _ => bad(format!("'{k}' must be an integer")),
    }
}

fn opt_dimens(a: &Args, k: &str) -> R<Dimens> {
    let m = as_map(a.req(k)?, k)?;
    let sp = |n: &str| -> R<i64> {
        let v = m.int_req(n)?;
        if v.abs() > MAX_DIMEN_SP { return bad(format!("dimens.{n} is out of range (at most 2^30 sp)")) }
        Ok(v)
    };
    let d = Dimens {
        paperwidth: sp("paperwidth")?, paperheight: sp("paperheight")?, textwidth: sp("textwidth")?,
        textheight: sp("textheight")?, oddsidemargin: sp("oddsidemargin")?, evensidemargin: sp("evensidemargin")?,
        topmargin: sp("topmargin")?, headheight: sp("headheight")?, headsep: sp("headsep")?, footskip: sp("footskip")?,
        hoffset: sp("hoffset")?,
    };
    if d.paperwidth <= 0 || d.paperheight <= 0 { return bad("the paper has no size") }
    if d.paperwidth > MAX_PAPER_SP || d.paperheight > MAX_PAPER_SP { return bad("the paper is larger than 150 cm") }
    if d.textwidth <= 0 || d.textheight <= 0 { return bad("the margins leave no room for the text") }
    Ok(d)
}

fn page_args(a: &Args) -> R<PageArgs> {
    let mut p = PageArgs::default();
    p.page = a.int_req("page")?;
    if p.page < 1 || p.page > 1_000_000 { return bad("the page number must be between 1 and 1000000") }
    p.twoside = a.flag("twoside")?.unwrap_or(false);
    p.dimens = opt_dimens(a, "dimens")?;
    p.seed = a.int("seed")?;
    p.mode = match a.text("mode")?.unwrap_or("grown") {
        "grown" => Mode::Grown,
        "authored" => Mode::Authored,
        m => return bad(format!("mode is 'authored' or 'grown', not '{m}'")),
    };
    p.theme = a.text("theme")?.map(str::to_string);
    p.season = a.num("age")?;
    if let Some(v) = a.get("marks") {
        let Value::Array(items) = v else { return bad("'marks' must be an array") };
        if items.len() > MAX_MARKS { return bad("too many marks") }
        for it in items {
            let m = as_map(it, "a mark")?;
            let y = m.num_req("y")?;
            let kind = match m.text_req("kind")? {
                "heading" => {
                    let l = m.int_req("arg")?;
                    MarkKind::Heading(i32::try_from(l).or_else(|_| bad("heading level out of range"))?)
                }
                "bloom" => MarkKind::Bloom(m.text("arg")?.unwrap_or("").to_string()),
                k => return bad(format!("a mark is a 'heading' or a 'bloom', not '{k}'")),
            };
            p.marks.push(Mark { kind, y });
        }
    }
    p.density = a.num("density")?.unwrap_or(1.0);
    p.intensity = a.num("intensity")?.unwrap_or(1.0);
    p.visitors = a.flag("visitors")?.unwrap_or(true);
    if let Some(s) = a.text("ink")?.filter(|s| !s.is_empty()) { p.ink = s.into() }
    if let Some(s) = a.text("accent")?.filter(|s| !s.is_empty()) { p.accent = s.into() }
    p.spill = a.num("spilly")?;
    if let Some(v) = a.get("frame") {
        let m = as_map(v, "frame")?;
        let (k, n) = (m.int_req("k")?, m.int_req("n")?);
        if n < 1 || n > 1_000_000 || k < 1 || k > n { return bad(format!("frame {k} of {n}: k must be between 1 and n")) }
        p.frame = Some(Frame { k, n, explain: m.flag("explain")?.unwrap_or(false) });
    }
    p.nohead = a.flag("nohead")?.unwrap_or(false);
    p.background = a.text("background")?.filter(|s| !s.is_empty()).map(str::to_string);
    Ok(p)
}

fn hex(a: &Args, k: &str) -> R<String> {
    let s = a.text(k)?.unwrap_or("");
    if !s.is_empty() { svg::html(s)?; }
    Ok(s.to_string())
}

fn color(a: &Args, k: &str) -> R<Option<Color>> {
    match a.text(k)? {
        None | Some("") => Ok(None),
        Some(s) => svg::html(s).map(Some),
    }
}

fn opts(v: Option<&Value>) -> R<Opts> {
    let Some(v) = v else { return Ok(Opts::default()) };
    let a = as_map(v, "opts")?;
    for (k, _) in a.0 {
        let Some(k) = k.as_text() else { return bad("opts keys must be strings") };
        const KNOWN: [&str; 25] = ["tone", "w", "eye", "color", "bend", "trail", "side", "open", "fork", "facing", "depth",
            "tilt", "streaks", "star", "shape", "rays", "petals", "lobes", "heart", "drift", "lift", "style", "k", "p", "v"];
        if !KNOWN.contains(&k) { return bad(format!("unknown option '{k}'")) }
    }
    let n = |k: &str| a.f64_in(k, MAX_OPT);
    let capped = |k: &str, hi: f64| -> R<Option<f64>> {
        let x = a.f64_in(k, MAX_OPT)?;
        if let Some(x) = x { if x < 0.0 || x > hi { return bad(format!("'{k}' must be between 0 and {hi}")) } }
        Ok(x)
    };
    let list = |k: &str| -> R<Option<Vec<f64>>> {
        match a.get(k) {
            None => Ok(None),
            Some(Value::Array(xs)) => {
                if xs.len() > 256 { return bad(format!("'{k}' has too many values")) }
                xs.iter().map(|x| number(x, k, MAX_OPT)).collect::<R<Vec<_>>>().map(Some)
            }
            Some(_) => bad(format!("'{k}' must be an array of numbers")),
        }
    };
    Ok(Opts {
        tone: n("tone")?, w: n("w")?, eye: color(&a, "eye")?, color: color(&a, "color")?, bend: n("bend")?,
        trail: a.flag("trail")?, side: n("side")?, open: n("open")?, fork: a.flag("fork")?, facing: n("facing")?,
        depth: capped("depth", MAX_DEPTH_OPT)?, tilt: n("tilt")?, streaks: capped("streaks", MAX_COUNT_OPT)?, star: a.flag("star")?,
        shape: a.text("shape")?.map(str::to_string), rays: a.flag("rays")?, petals: capped("petals", MAX_COUNT_OPT)?, lobes: capped("lobes", MAX_COUNT_OPT)?,
        heart: a.flag("heart")?, drift: n("drift")?, lift: n("lift")?, style: n("style")?, k: n("k")?,
        p: list("p")?, v: list("v")?,
    })
}

// ---------------------------------------------------------------- encoding
fn text(s: &str) -> Value { Value::Text(s.into()) }
fn flt(x: f64) -> Value { Value::Float(x) }
fn map(kv: Vec<(&str, Value)>) -> Value { Value::Map(kv.into_iter().map(|(k, v)| (text(k), v)).collect()) }

fn encode(v: &Value) -> R<Vec<u8>> {
    let mut b = vec![];
    ciborium::into_writer(v, &mut b).or_else(|e| bad(format!("cannot encode the result: {e}")))?;
    Ok(b)
}

// ---------------------------------------------------------------- canvases
/// An SVG canvas for streams drawn around the origin: the box of every coordinate of every
/// path (control points included) plus a margin for the strokes, in pt (= bp), with the
/// origin's offset from the canvas's top-left corner.
struct Canvas { svg: String, width: f64, height: f64, dx: f64, dy: f64 }

fn canvas(streams: &[String]) -> R<Canvas> {
    let joined = streams.join("\n");
    let (mut x0, mut y0, mut x1, mut y1) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    let mut nums: Vec<f64> = vec![];
    let mut wmax = 0f64;
    for t in joined.split_ascii_whitespace() {
        let c = t.as_bytes()[0];
        if c.is_ascii_alphabetic() {
            if t == "w" { if let Some(w) = nums.last() { wmax = wmax.max(*w) } }
            let take = match t { "m" | "l" => 2, "c" => 6, _ => 0 };
            if take > 0 && nums.len() >= take {
                for p in nums[nums.len() - take..].chunks(2) {
                    x0 = x0.min(p[0]); x1 = x1.max(p[0]); y0 = y0.min(p[1]); y1 = y1.max(p[1]);
                }
            }
            nums.clear();
        } else if let Ok(x) = t.parse::<f64>() {
            if !x.is_finite() { return bad("a drawing has a non-finite coordinate") }
            nums.push(x);
        }
    }
    if !x0.is_finite() || !y0.is_finite() {
        return Ok(Canvas { svg: "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"0\" height=\"0\"/>\n".into(), width: 0.0, height: 0.0, dx: 0.0, dy: 0.0 });
    }
    if !(x1.is_finite() && y1.is_finite() && wmax.is_finite()) { return bad("a drawing has a non-finite extent") }
    let pad = f64::max(2.0, wmax / 2.0 + 1.0);
    let (l, r, top, bottom) = ((x0 - pad).floor(), (x1 + pad).ceil(), (y1 + pad).ceil(), (y0 - pad).floor());
    let (w, h) = (r - l, top - bottom);
    let mut parts = vec![];
    svg::translate(&joined, &mut parts, "o")?;
    let mut o = String::from("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"");
    fmt::fixed(&mut o, w, 0);
    o.push_str("\" height=\"");
    fmt::fixed(&mut o, h, 0);
    o.push_str("\" viewBox=\"");
    for (k, v) in [l, -top, w, h].iter().enumerate() {
        if k > 0 { o.push(' ') }
        fmt::fixed(&mut o, *v, 0);
    }
    o.push_str("\"><g transform=\"matrix(1 0 0 -1 0 0)\" stroke-linecap=\"round\" stroke-linejoin=\"round\">\n");
    o.push_str(&parts.join("\n"));
    o.push_str("\n</g></svg>\n");
    Ok(Canvas { svg: o, width: w, height: h, dx: -l, dy: top })
}

fn canvas_value(c: Canvas, extra: bool) -> Value {
    let mut kv = vec![("svg", Value::Bytes(c.svg.into_bytes())), ("width", flt(c.width)), ("height", flt(c.height))];
    if extra { kv.push(("dx", flt(c.dx))); kv.push(("dy", flt(c.dy))) }
    map(kv)
}

// ---------------------------------------------------------------- the four functions
pub fn page_cbor(bytes: &[u8]) -> R<Vec<u8>> {
    let v = parse(bytes)?;
    let a = as_map(&v, "the arguments")?;
    let mut p = page_args(&a)?;
    p.ink = if p.ink.is_empty() { "2456C4".into() } else { p.ink };
    for (k, x) in [("density", p.density), ("intensity", p.intensity)] {
        if !x.is_finite() { return bad(format!("{k} must be a finite number")) }
    }
    if p.mode == Mode::Grown {
        for m in &p.marks {
            if let MarkKind::Bloom(arg) = &m.kind {
                if arg.is_empty() { continue }
                crate::grow::theme(p.theme.as_deref())?;
                if crate::grow::motif(arg).is_none() { return bad(format!("no motif '{arg}'")) }
            }
        }
    }
    let out = svg::page(&p)?;
    let g = &out.geometry;
    let ph = g.ph;
    let captions = out.captions.iter().map(|c| {
        map(vec![
            ("x", flt(c.x_cm * 72.0 / 2.54)), ("y", flt((ph - c.y_cm) * 72.0 / 2.54)),
            ("anchor", text(&c.anchor)), ("size", flt(c.size_pt)), ("rotate", flt(c.rotate)),
            ("color", text(&format!("#{:02x}{:02x}{:02x}", c.color[0], c.color[1], c.color[2]))),
            ("opacity", flt(c.opacity)), ("markup", text(&c.markup)),
        ])
    }).collect();
    encode(&map(vec![
        ("svg", Value::Bytes(out.svg.into_bytes())), ("captions", Value::Array(captions)),
        ("mirror", Value::Bool(out.geometry.mirror)), ("stats", text(&out.stats)),
        ("geometry", map(vec![
            ("pw", flt(g.pw)), ("ph", flt(g.ph)), ("textleft", flt(g.textleft)), ("textright", flt(g.textright)),
            ("ytop", flt(g.ytop)), ("ybot", flt(g.ybot)), ("colright", flt(g.colright)),
            ("headrule", flt(g.headrule)), ("footy", flt(g.footy)),
        ])),
    ]))
}

fn ink_accent(a: &Args) -> R<(String, String)> { Ok((hex(a, "ink")?, hex(a, "accent")?)) }

pub fn ornament_cbor(bytes: &[u8]) -> R<Vec<u8>> {
    let v = parse(bytes)?;
    let a = as_map(&v, "the arguments")?;
    let name = a.text_req("name")?;
    let size = a.int_req("size")?;
    if size <= 0 || size > MAX_BOX_SP { return bad("the ornament's size must be positive and at most 100000000 sp") }
    let angle = a.f64_in("angle", 1e6)?.unwrap_or(0.0);
    let count = a.int("count")?.unwrap_or(1);
    if count < 1 { return bad("the ornament count starts at 1") }
    let (ink, acc) = ink_accent(&a)?;
    let s = glue::ornament_streams(name, size, &ink, &acc, angle, a.int("seed")?, count, a.text("theme")?)?;
    encode(&canvas_value(canvas(&s)?, true))
}

pub fn draw_cbor(bytes: &[u8]) -> R<Vec<u8>> {
    let v = parse(bytes)?;
    let a = as_map(&v, "the arguments")?;
    let name = a.text_req("name")?;
    let size = a.num_req("size")?;
    if size <= 0.0 || size > MAX_DRAW_CM { return bad("the size must be positive (cm) and at most 50") }
    let o = opts(a.get("opts"))?;
    if matches!(name, "koch" | "hilbert") {
        if let Some(l) = o.p.as_ref().and_then(|p| p.get(1)) {
            if *l < 0.0 || *l > MAX_LEVEL_OPT { return bad(format!("{name}: p[1] must be between 0 and 7")) }
        }
    }
    let (ink, acc) = ink_accent(&a)?;
    let seed = a.int_req("seed")?;
    let theme = a.text("theme")?;
    let s = if o.lift.is_some() { glue::draw_ghost_streams(name, size, &o, &ink, &acc, theme, seed)? }
        else { glue::draw_streams(name, size, &o, &ink, &acc, theme, seed)? };
    encode(&canvas_value(canvas(&s)?, true))
}

pub fn initial_cbor(bytes: &[u8]) -> R<Vec<u8>> {
    let v = parse(bytes)?;
    let a = as_map(&v, "the arguments")?;
    let letter = a.text_req("letter")?;
    let side = a.int_req("side")?;
    if side <= 0 || side > MAX_BOX_SP { return bad("the initial's side must be positive and at most 100000000 sp") }
    let (hang, cap) = (a.int("hang")?.unwrap_or(0), a.int("cap")?.unwrap_or(side));
    if hang.abs() > MAX_DIMEN_SP || cap.abs() > MAX_DIMEN_SP { return bad("hang or cap out of range") }
    let font = match a.req("font")? { Value::Bytes(b) => b, _ => return bad("'font' must be bytes") };
    let shapes = initial::shapes_from_font(font).map_err(|e| format!("florilegium: {e}"))?;
    let s = glue::initial_streams(letter, side, hang, cap, &shapes, a.int_req("seed")?)?;
    encode(&canvas_value(canvas(&s)?, true))
}

// ---------------------------------------------------------------- the exports
#[cfg(feature = "plugin")]
mod exports {
    use wasm_minimal_protocol::{initiate_protocol, wasm_func};

    initiate_protocol!();

    #[wasm_func]
    pub fn page(args: &[u8]) -> Result<Vec<u8>, String> { super::page_cbor(args) }
    #[wasm_func]
    pub fn ornament(args: &[u8]) -> Result<Vec<u8>, String> { super::ornament_cbor(args) }
    #[wasm_func]
    pub fn draw(args: &[u8]) -> Result<Vec<u8>, String> { super::draw_cbor(args) }
    #[wasm_func]
    pub fn initial(args: &[u8]) -> Result<Vec<u8>, String> { super::initial_cbor(args) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_drawing_is_a_zero_canvas() {
        let c = canvas(&[]).unwrap();
        assert_eq!((c.width, c.height, c.dx, c.dy), (0.0, 0.0, 0.0, 0.0));
        assert!(c.svg.contains("width=\"0\""));
    }

    #[test]
    fn canvas_pads_by_the_stroke_and_rejects_non_finite() {
        let c = canvas(&["q 10 w 0 0 m 10 0 l S Q".to_string()]).unwrap();
        assert_eq!((c.dx, c.dy), (6.0, 6.0));
        assert!(canvas(&["-1e999 0 m".to_string()]).is_err());
    }
}
