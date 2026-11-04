// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! florilegium-svg.lua: a florilegium page as SVG, without TeX. ink.rs writes PDF content streams as
//! always and `translate` reads them back (the few operators ink uses) and writes SVG
//! elements; `page` lays the page out as LuaLaTeX and the geometry package would and runs
//! the glue (glue.rs) on it. Lengths are scanned as TeX scans them (`sp`).
//!
//! Differences from the Lua in types only: `translate` appends its elements to a
//! `Vec<String>` (the Lua's list, joined with "\n" at the end), and `Dimens::from_margins`
//! reports the Lua's "no room for the text" error.
use crate::fmt;
use crate::glue::{self, Dimens, Geometry, Mode, PageArgs, PlacedCaption};
use crate::ink::{Color, CM};

// ---------------------------------------------------------------- lengths
const UNITS: [(&str, i128, i128); 5] = [("pt", 1, 1), ("mm", 7227, 2540), ("cm", 7227, 254), ("in", 7227, 100), ("bp", 7227, 7200)];

/// a length as TeX scans it ("68mm" -> 12679822 sp): integer part, the fraction rounded to
/// 2^-16, then the unit as the ratio num/denom with the remainder carried, as in TeX's
/// scan_dimen. A bare number is millimeters.
pub fn sp(s: &str) -> Result<i64, String> {
    let not = || format!("florilegium: not a length: '{s}'");
    let b = s.as_bytes();
    let is_space = |c: u8| matches!(c, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r');
    let mut i = 0;
    while i < b.len() && is_space(b[i]) { i += 1 }
    let mut negative = false;
    if i < b.len() && (b[i] == b'-' || b[i] == b'+') { negative = b[i] == b'-'; i += 1 }
    let a = i;
    while i < b.len() && b[i].is_ascii_digit() { i += 1 }
    let int = &s[a..i];
    if i < b.len() && b[i] == b'.' { i += 1 }
    let a = i;
    while i < b.len() && b[i].is_ascii_digit() { i += 1 }
    let frac = &s[a..i];
    while i < b.len() && is_space(b[i]) { i += 1 }
    let a = i;
    while i < b.len() && b[i].is_ascii_alphabetic() { i += 1 }
    let unit = &s[a..i];
    while i < b.len() && is_space(b[i]) { i += 1 }
    if i != b.len() || format!("{int}{frac}").is_empty() { return Err(not()) }
    let (_, un, ud) = *UNITS.iter().find(|u| u.0 == if unit.is_empty() { "mm" } else { unit })
        .ok_or_else(|| format!("florilegium: unknown unit '{unit}' in '{s}'"))?;
    let mut v: i128 = if int.is_empty() { 0 } else { int.parse().map_err(|_| not())? };
    let mut f: i128 = 0;
    if !frac.is_empty() {
        let mut a: i128 = 0;
        for k in (1..=frac.len().min(17)).rev() {
            a = (a + (frac.as_bytes()[k - 1] - b'0') as i128 * 131072).div_euclid(10);
        }
        f = (a + 1).div_euclid(2);
    }
    if un != 1 {
        let q = (v * un).div_euclid(ud);
        let r = (v * un).rem_euclid(ud);
        f = (un * f + 65536 * r).div_euclid(ud);
        v = q + f.div_euclid(65536);
        f = f.rem_euclid(65536);
    }
    let n = i64::try_from(v * 65536 + f).map_err(|_| not())?;
    Ok(if negative { -n } else { n })
}

fn paper(name: &str) -> Option<(&'static str, &'static str)> {
    Some(match name {
        "a4" => ("210mm", "297mm"),
        "a5" => ("148mm", "210mm"),
        "letter" => ("8.5in", "11in"),
        "legal" => ("8.5in", "14in"),
        _ => return None,
    })
}

// ---------------------------------------------------------------- colors
/// TeX's print_scaled, without "pt"
fn print_scaled(s: i64) -> String {
    let mut out = format!("{}.", s.div_euclid(65536));
    let mut delta: i64 = 10;
    let mut s = 10 * s.rem_euclid(65536) + 5;
    loop {
        if delta > 65536 { s = s + 0x8000 - 50000 }
        out.push_str(&s.div_euclid(65536).to_string());
        s = 10 * s.rem_euclid(65536);
        delta *= 10;
        if s <= delta { break }
    }
    out
}

/// n/255 as xcolor hands it to Lua: five decimals, the sixth rounding up by 1 sp, read as a
/// TeX length and printed back as TeX prints one (0x24 is 0.14119, not 0.141176)
fn xcolor(n: i64) -> Result<f64, String> {
    let (q, mut r) = (n / 255, n % 255);
    let mut digits = [0i64; 6];
    for d in digits.iter_mut() { r *= 10; *d = r / 255; r %= 255 }
    let digits5: String = digits[..5].iter().map(|d| d.to_string()).collect();
    let mut s = sp(&format!("{q}.{digits5}pt"))?;
    if digits[5] > 4 { s += 1 }
    let p = print_scaled(s);
    let p = p.strip_suffix(".0").unwrap_or(&p);
    p.parse().map_err(|_| "florilegium: bad color".to_string())
}

/// an HTML color (RRGGBB, an optional leading #) as xcolor hands it to Lua
pub fn html(h: &str) -> Result<Color, String> {
    let h = h.strip_prefix('#').unwrap_or(h);
    if h.len() != 6 || !h.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("florilegium: not a color (RRGGBB): '{h}'"));
    }
    let c = |a: usize| xcolor(i64::from_str_radix(&h[a..a + 2], 16).unwrap());
    Ok([c(0)?, c(2)?, c(4)?])
}

// ---------------------------------------------------------------- PDF -> SVG
#[derive(Clone)]
struct St<'a> { fill: [u8; 7], stroke: [u8; 7], w: &'a str, dash: Option<String>, op: f64, mult: bool }

enum A<'a> { S(&'a str), Arr(Vec<&'a str>) }

fn hex(r: &str, g: &str, b: &str) -> Result<[u8; 7], String> {
    let c = |v: &str| -> Result<u8, String> {
        let x: f64 = v.parse().map_err(|_| format!("florilegium: florilegium-svg.lua: bad number '{v}'"))?;
        Ok(((x * 255.0 + 0.5).floor() as i64).clamp(0, 255) as u8)
    };
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut o = *b"#000000";
    for (k, v) in [c(r)?, c(g)?, c(b)?].into_iter().enumerate() {
        o[1 + 2 * k] = H[(v >> 4) as usize];
        o[2 + 2 * k] = H[(v & 15) as usize];
    }
    Ok(o)
}

fn num(t: &str) -> &str { t.strip_suffix(".0").unwrap_or(t) }

fn color(c: &[u8; 7]) -> &str { std::str::from_utf8(c).unwrap() }

// ---------------------------------------------------------------- the literals
pub fn translate(stream: &str, out: &mut Vec<String>, id: &str) -> Result<(), String> {
    let mut st = St { fill: *b"#000000", stroke: *b"#000000", w: "1", dash: None, op: 1.0, mult: false };
    let mut stack: Vec<(St, usize)> = vec![];
    let mut open = 0usize;
    let mut args: Vec<A> = vec![];
    let mut d = String::new();
    let mut arr: Option<Vec<&str>> = None;
    let mut nclip = 0;
    let mut clip = false;
    let bad = || "florilegium: florilegium-svg.lua: malformed content stream".to_string();
    for t in stream.split_ascii_whitespace() {
        let c = t.as_bytes()[0];
        if arr.is_some() || c == 91 {
            let piece = if arr.is_some() { t } else { arr = Some(vec![]); &t[1..] };
            let (v, closed) = match piece.strip_suffix(']') { Some(v) => (v, true), None => (piece, false) };
            if !v.is_empty() { arr.as_mut().unwrap().push(num(v)) }
            if closed { args.push(A::Arr(arr.take().unwrap())) }
        } else if c.is_ascii_digit() || c == b'-' || c == b'.' || c == b'/' {
            args.push(A::S(t));
        } else {
            let n = args.len();
            let s = |k: usize| -> Result<&str, String> {
                match n.checked_sub(k).and_then(|i| args.get(i)) { Some(A::S(s)) => Ok(s), _ => Err(bad()) }
            };
            match t {
                "m" | "l" => {
                    d.push(if t == "m" { 'M' } else { 'L' });
                    d.push_str(num(s(2)?));
                    d.push(' ');
                    d.push_str(num(s(1)?));
                }
                "c" => {
                    d.push('C');
                    for k in (1..=6).rev() {
                        d.push_str(num(s(k)?));
                        if k > 1 { d.push(' ') }
                    }
                }
                "h" => d.push('Z'),
                "f" | "S" => {
                    let mut a = String::with_capacity(d.len() + 120);
                    a.push_str("<path d=\"");
                    a.push_str(&d);
                    a.push('"');
                    if t == "S" {
                        a.push_str(" fill=\"none\" stroke=\"");
                        a.push_str(color(&st.stroke));
                        a.push_str("\" stroke-width=\"");
                        a.push_str(st.w);
                        a.push('"');
                        if let Some(dash) = &st.dash { a.push_str(" stroke-dasharray=\""); a.push_str(dash); a.push('"') }
                        if st.op < 1.0 { a.push_str(" stroke-opacity=\""); fmt::g(&mut a, st.op); a.push('"') }
                    } else {
                        a.push_str(" fill=\"");
                        a.push_str(color(&st.fill));
                        a.push('"');
                        if st.op < 1.0 { a.push_str(" fill-opacity=\""); fmt::g(&mut a, st.op); a.push('"') }
                    }
                    if st.mult { a.push_str(" style=\"mix-blend-mode:multiply\"") }
                    a.push_str("/>");
                    out.push(a);
                    d.clear();
                }
                "W" => clip = true,
                "n" => {
                    if clip {
                        nclip += 1;
                        let cid = format!("{id}c{nclip}");
                        out.push(format!("<clipPath id=\"{cid}\"><path d=\"{d}\"/></clipPath><g clip-path=\"url(#{cid})\">"));
                        open += 1;
                        clip = false;
                    }
                    d.clear();
                }
                "q" => { stack.push((st.clone(), open)); open = 0 }
                "Q" => {
                    for _ in 0..open { out.push("</g>".into()) }
                    let (s0, o0) = stack.pop().ok_or_else(bad)?;
                    st = s0;
                    open = o0;
                }
                "rg" => st.fill = hex(s(3)?, s(2)?, s(1)?)?,
                "RG" => st.stroke = hex(s(3)?, s(2)?, s(1)?)?,
                "w" => st.w = num(s(1)?),
                "d" => {
                    st.dash = match n.checked_sub(2).and_then(|i| args.get(i)) {
                        Some(A::Arr(a)) if !a.is_empty() => Some(a.join(" ")),
                        _ => None,
                    };
                }
                "gs" => {
                    let name = s(1)?;
                    let parsed = name.strip_prefix("/flo").and_then(|r| {
                        let kind = r.chars().next().filter(|c| matches!(c, 'a' | 'm'))?;
                        let v = &r[1..];
                        (!v.is_empty() && v.bytes().all(|c| c.is_ascii_digit())).then(|| (kind == 'm', v.parse::<f64>().unwrap_or(0.0)))
                    });
                    let (mult, v) = parsed.ok_or_else(|| format!("florilegium: unknown graphics state {name}"))?;
                    st.op = v / 100.0;
                    st.mult = mult;
                }
                "J" | "j" => {}
                _ => return Err(format!("florilegium: florilegium-svg.lua cannot translate the PDF operator '{t}'")),
            }
            args.clear();
        }
    }
    for _ in 0..open { out.push("</g>".into()) }
    Ok(())
}

/// the streams of a few primitives as a standalone SVG sized to their points, as
/// tools/ornament.lua writes them
pub fn standalone(streams: &[String]) -> Result<String, String> {
    let s = streams.join("\n");
    let b = s.as_bytes();
    let (mut x0, mut y0, mut x1, mut y1) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    // gmatch("(%-?[%d.]+) (%-?[%d.]+) [mlc]")
    let number = |mut i: usize| -> Option<usize> {
        if i < b.len() && b[i] == b'-' { i += 1 }
        let a = i;
        while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'.') { i += 1 }
        (i > a).then_some(i)
    };
    let mut i = 0;
    while i < b.len() {
        let m = (|| {
            let e1 = number(i)?;
            if b.get(e1) != Some(&b' ') { return None }
            let e2 = number(e1 + 1)?;
            if b.get(e2) != Some(&b' ') || !matches!(b.get(e2 + 1), Some(b'm' | b'l' | b'c')) { return None }
            Some((&s[i..e1], &s[e1 + 1..e2], e2 + 2))
        })();
        match m {
            Some((xs, ys, end)) => {
                let (x, y): (f64, f64) = (xs.parse().map_err(|_| "florilegium: bad number")?, ys.parse().map_err(|_| "florilegium: bad number")?);
                x0 = x0.min(x); x1 = x1.max(x); y0 = y0.min(y); y1 = y1.max(y);
                i = end;
            }
            None => i += 1,
        }
    }
    let pad = 1.0;
    let mut parts = vec![];
    translate(&s, &mut parts, "o")?;
    let mut o = String::from("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"");
    for (k, v) in [x0 - pad, -y1 - pad, x1 - x0 + 2.0 * pad, y1 - y0 + 2.0 * pad].iter().enumerate() {
        if k > 0 { o.push(' ') }
        fmt::fixed(&mut o, *v, 1);
    }
    o.push_str("\"><g transform=\"matrix(1 0 0 -1 0 ");
    fmt::fixed(&mut o, 0.0, 1);
    o.push_str(")\" stroke-linecap=\"round\" stroke-linejoin=\"round\">\n");
    o.push_str(&parts.join("\n"));
    o.push_str("\n</g></svg>\n");
    Ok(o)
}

// ---------------------------------------------------------------- the page
#[derive(Debug)]
pub struct PageOut {
    pub svg: String,
    pub captions: Vec<PlacedCaption>,
    pub stats: String,
    pub geometry: Geometry,
}

fn f2(o: &mut String, x: f64) { fmt::fixed(o, x, 2) }

/// the page as SVG: big points, y up as in PDF, so the paths are copied as they are
pub fn page(a: &PageArgs) -> Result<PageOut, String> {
    let garden = glue::garden(a)?;
    let mut streams = garden.streams;
    if a.nohead { streams.pop(); }
    let g = &garden.geometry;
    let (w, h) = (g.pw * CM, g.ph * CM);
    let id = a.id.clone().unwrap_or_else(|| format!("flo{}-", a.seed.unwrap_or(0)));
    let mut out: Vec<String> = vec![];
    let mut head = String::from("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"");
    let b = a.bleed.unwrap_or(0.0);
    f2(&mut head, (g.pw + 2.0 * b) * 10.0);
    head.push_str("mm\" height=\"");
    f2(&mut head, (g.ph + 2.0 * b) * 10.0);
    head.push_str("mm\" viewBox=\"");
    if a.bleed.is_some() {
        f2(&mut head, -b * CM);
        head.push(' ');
        f2(&mut head, -b * CM);
        head.push(' ');
        f2(&mut head, w + 2.0 * b * CM);
        head.push(' ');
        f2(&mut head, h + 2.0 * b * CM);
    } else {
        head.push_str("0 0 ");
        f2(&mut head, w);
        head.push(' ');
        f2(&mut head, h);
    }
    head.push_str("\">");
    out.push(head);
    if let Some(bg) = &a.background {
        let hx = bg.strip_prefix('#').unwrap_or(bg);
        if hx.len() != 6 || !hx.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("florilegium: not a color (RRGGBB): '{hx}'"));
        }
        let mut r = String::from("<rect width=\"");
        f2(&mut r, w);
        r.push_str("\" height=\"");
        f2(&mut r, h);
        r.push_str(&format!("\" fill=\"#{}\"/>", hx.to_ascii_lowercase()));
        out.push(r);
    }
    let mut gt = String::from("<g transform=\"matrix(1 0 0 -1 0 ");
    f2(&mut gt, h);
    gt.push_str(")\" stroke-linecap=\"round\" stroke-linejoin=\"round\">");
    out.push(gt);
    for (i, s) in streams.iter().enumerate() { translate(s, &mut out, &format!("{id}{}", i + 1))? }
    out.push("</g></svg>\n".into());
    Ok(PageOut { svg: out.join("\n"), captions: garden.captions, stats: garden.stats, geometry: garden.geometry })
}

// ---------------------------------------------------------------- command line
pub const USAGE: &str = "usage: florilegium-svg [options] > page.svg

  --seed N          the document's seed (\\def\\florilegiumseed{N}); default: none
  --page N          the page number (default 1)
  --paper NAME      a4 (default), a5, letter, legal, or WxH in mm (e.g. 176x250)
  --left/--right/--top/--bottom LEN   margins as geometry takes them (default 68mm 22mm 32mm 30mm)
  --headsep LEN     (default 8mm)    --headheight LEN (12pt)    --footskip LEN (30pt)
  --twoside         a two-sided document: the garden in the wider outer margin
  --textend LEN     the last page: the text ends LEN below the top of the text block
  --season S        spring, summer (default), autumn, or 0..1
  --theme NAME      flora (default), geometric or math (math: its text needs TeX, left out)
  --density X       0.05..1 (default 1)      --intensity X   0..1 (default 1)
  --ink RRGGBB      (default 2456C4)         --accent RRGGBB (default B4662F)
  --novisitors      no butterflies, bees, snails or birds
  --nohead          leave out the head rule and its ivy leaf
  --authored        the hand-placed composition (A4, 68 mm left margin)
  --background RRGGBB   paint the paper (default: transparent)
  --bleed LEN       let the garden run LEN past the paper's edges, uncut; the SVG grows by LEN
                    on every side (for a garden set on a page of its own, as in an e-book)
  -o FILE           write to FILE instead of standard output
A length is a number and a unit (mm, cm, in, pt, bp); a bare number is mm.
";

pub struct Cli { pub page: PageArgs, pub out: Option<String>, pub help: bool }

fn tonumber(s: &str) -> Option<f64> { s.trim().parse::<f64>().ok() }

/// florilegium-svg.lua's main(): the options of the command line as page arguments (`-h` and `-o` are
/// reported in `Cli`)
pub fn options(argv: &[String]) -> Result<Cli, String> {
    let flags = ["twoside", "novisitors", "nohead", "authored"];
    let values = ["seed", "page", "paper", "left", "right", "top", "bottom", "headsep", "headheight", "footskip", "textend",
        "season", "theme", "density", "intensity", "ink", "accent", "background",
        "bleed"];
    let mut o: std::collections::HashMap<&str, String> = Default::default();
    let (mut twoside, mut visitors, mut head, mut authored) = (false, true, true, false);
    let (mut file, mut help) = (None, false);
    let mut i = 0;
    while i < argv.len() {
        let a = argv[i].as_str();
        let key = a.strip_prefix("--").filter(|k| !k.is_empty() && k.bytes().all(|c| c.is_ascii_alphanumeric()));
        if a == "-h" || a == "--help" { help = true; break }
        else if a == "-o" { file = argv.get(i + 1).cloned(); i += 1 }
        else if let Some(k) = key.filter(|k| flags.contains(k)) {
            match k { "twoside" => twoside = true, "novisitors" => visitors = false, "nohead" => head = false, _ => authored = true }
        } else if let (Some(k), Some(v)) = (key.filter(|k| values.contains(k)), argv.get(i + 1)) {
            o.insert(values.iter().find(|x| **x == k).unwrap(), v.clone());
            i += 1;
        } else {
            return Err(format!("unknown option {a}\n{USAGE}"));
        }
        i += 1;
    }
    let mut pg = PageArgs::default();
    let len = |k: &str, default: &str| -> Result<i64, String> { sp(o.get(k).map(String::as_str).unwrap_or(default)) };
    let (pw, ph) = match o.get("paper").map(String::as_str).unwrap_or("a4") {
        p if paper(p).is_some() => { let (w, h) = paper(p).unwrap(); (sp(w)?, sp(h)?) }
        p => {
            let bad = || format!("unknown paper {p}");
            let (w, h) = p.split_once('x').ok_or_else(bad)?;
            let ok = |s: &str| !s.is_empty() && s.split('.').count() <= 2 && s.split('.').all(|x| x.bytes().all(|c| c.is_ascii_digit()))
                && !s.starts_with('.');
            if !ok(w) || !ok(h) { return Err(bad()) }
            (sp(&format!("{w}mm"))?, sp(&format!("{h}mm"))?)
        }
    };
    pg.dimens = Dimens::from_margins(pw, ph, len("left", "68mm")?, len("right", "22mm")?, len("top", "32mm")?, len("bottom", "30mm")?,
        len("headheight", "12pt")?, len("headsep", "8mm")?, len("footskip", "30pt")?)?;
    pg.page = tonumber(o.get("page").map(String::as_str).unwrap_or("1")).unwrap_or(1.0).floor() as i64;
    pg.twoside = twoside;
    if let Some(s) = o.get("seed") {
        match tonumber(s) {
            Some(x) if x >= 1.0 => pg.seed = Some(x.floor() as i64),
            _ => return Err("florilegium: a seed is a positive integer".into()),
        }
    }
    pg.mode = if authored { Mode::Authored } else { Mode::Grown };
    pg.theme = o.get("theme").filter(|t| !t.is_empty() && *t != "flora").cloned();
    pg.season = match o.get("season").map(String::as_str) {
        None | Some("summer") => None,
        Some("spring") => Some(0.0),
        Some("autumn") => Some(1.0),
        Some(s) => Some(tonumber(s).ok_or("florilegium: a season is spring, summer, autumn or 0..1")?),
    };
    pg.density = o.get("density").and_then(|s| tonumber(s)).unwrap_or(1.0);
    pg.intensity = o.get("intensity").and_then(|s| tonumber(s)).unwrap_or(1.0);
    pg.visitors = visitors;
    pg.nohead = !head;
    if let Some(s) = o.get("ink") { pg.ink = s.clone() }
    if let Some(s) = o.get("accent") { pg.accent = s.clone() }
    pg.background = o.get("background").cloned();
    if let Some(b) = o.get("bleed") { pg.bleed = Some(sp(b)? as f64 * 2.54 / (72.27 * 65536.0)) }
    if let Some(t) = o.get("textend") {
        let g = glue::geometry(&pg.dimens, pg.page, pg.twoside);
        pg.spill = Some(g.ytop - (sp(t)? as f64 * 2.54) / (72.27 * 65536.0));
    }
    Ok(Cli { page: pg, out: file, help })
}
