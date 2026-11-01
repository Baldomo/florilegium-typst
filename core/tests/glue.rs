// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
use florilegium_core::flora::Opts;
use florilegium_core::glue::{self, Dimens, Mode, PageArgs};
use florilegium_core::grow::{Mark, MarkKind};
use florilegium_core::{initial, svg};

fn a4() -> Dimens {
    let s = |x: &str| svg::sp(x).unwrap();
    Dimens::from_margins(s("210mm"), s("297mm"), s("68mm"), s("22mm"), s("32mm"), s("30mm"), s("12pt"), s("8mm"), s("30pt")).unwrap()
}

fn args() -> PageArgs { PageArgs { dimens: a4(), seed: Some(2), ..Default::default() } }

#[test]
fn lengths_and_colors() {
    assert_eq!(svg::sp("68mm").unwrap(), 12679822);
    assert_eq!(svg::sp("1in").unwrap(), 4736286);
    assert_eq!(svg::sp("12pt").unwrap(), 786432);
    assert_eq!(svg::sp(" -1.5 cm ").unwrap(), -svg::sp("1.5cm").unwrap());
    assert!(svg::sp("12 parsecs").unwrap_err().contains("unknown unit"));
    assert!(svg::sp("mm").unwrap_err().contains("not a length"));
    assert!(svg::sp("1.2.3").unwrap_err().contains("not a length"));
    assert_eq!(svg::html("2456C4").unwrap()[0], 0.14119);
    assert!(svg::html("2456C").unwrap_err().contains("not a color"));
}

#[test]
fn geometry_mirrors_the_outer_margin() {
    let d = a4();
    let odd = glue::geometry(&d, 1, true);
    assert!(!odd.mirror);
    let even = glue::geometry(&d, 2, true);
    assert!(even.mirror);
    // the layout of the mirrored page is the layout of the odd one; only the drawing is reflected
    assert!((even.textleft - odd.textleft).abs() < 1e-6);
    assert!(!glue::geometry(&d, 2, false).mirror);
    assert!(Dimens::from_margins(100, 100, 60, 60, 0, 0, 0, 0, 0).is_err());
}

#[test]
fn seeds_and_age() {
    assert_eq!(glue::seed(3, None), 3 * 97 + 1030);
    assert_eq!(glue::seed(3, Some(7)), 7 * 7919 + 3 * 131);
    assert_eq!(glue::auto_age(1, 1), 0.5);
    assert_eq!(glue::auto_age(1, 5), 0.0);
    assert_eq!(glue::auto_age(3, 5), 0.5);
    assert_eq!(glue::auto_age(9, 5), 1.0);
}

#[test]
fn marks_pass_through() {
    florilegium_core::reset();
    let plain = glue::garden(&args()).unwrap();
    florilegium_core::reset();
    let mut a = args();
    a.marks = vec![Mark { kind: MarkKind::Heading(1), y: 20.0 }, Mark { kind: MarkKind::Bloom("umbel".into()), y: 9.0 }];
    let marked = glue::garden(&a).unwrap();
    assert_ne!(plain.streams, marked.streams);
}

#[test]
fn errors_leave_nothing_behind() {
    florilegium_core::reset();
    let first = svg::page(&args()).unwrap().svg;
    let mut bad = args();
    bad.theme = Some("nosuch".into());
    assert!(svg::page(&bad).unwrap_err().contains("nosuch"));
    bad = args();
    bad.ink = "blue".into();
    assert!(svg::page(&bad).unwrap_err().contains("not a color"));
    bad = args();
    bad.seed = Some(0);
    assert!(svg::page(&bad).unwrap_err().contains("positive integer"));
    bad = args();
    bad.background = Some("zzz".into());
    assert!(svg::page(&bad).unwrap_err().contains("not a color"));
    assert!(glue::draw("nosuch", 1.0, &Opts::default(), "", "", None, 1).unwrap_err().contains("nosuch"));
    assert!(glue::ornament("nosuch", 786432, "", "", 0.0, None, 1, None).is_err());
    assert!(glue::ornament("flower", 786432, "xyz", "", 0.0, None, 1, None).is_err());
    assert!(glue::draw("flower", 1.0, &Opts::default(), "", "", Some("nosuch"), 1).is_err());
    florilegium_core::reset();
    assert_eq!(svg::page(&args()).unwrap().svg, first);
    // without the reset of the test: the failed calls reset the state themselves
    assert_eq!(svg::page(&args()).unwrap().svg, first);
}

#[test]
fn ornament_colors_are_restored() {
    florilegium_core::reset();
    let a = glue::ornament("flower", 786432, "", "", 0.0, Some(1), 1, None).unwrap();
    let _ = glue::ornament("flower", 786432, "5B2A1F", "2F6B3A", 0.0, Some(1), 1, None).unwrap();
    assert_eq!(glue::ornament("flower", 786432, "", "", 0.0, Some(1), 1, None).unwrap(), a);
}

#[test]
fn captions_are_placed_and_converted() {
    florilegium_core::reset();
    let mut a = args();
    a.theme = Some("math".into());
    a.twoside = true;
    a.page = 2;
    let g = glue::garden(&a).unwrap();
    assert!(g.geometry.mirror);
    assert!(!g.captions.is_empty());
    for c in &g.captions {
        assert!(!c.tex.is_empty() && c.x_cm >= 0.0 && c.x_cm <= g.geometry.pw);
        assert!(!c.markup.contains("\\frac"), "{}", c.markup);
    }
}

#[test]
fn authored_and_frame_modes() {
    florilegium_core::reset();
    let mut a = args();
    a.mode = Mode::Authored;
    let g = glue::garden(&a).unwrap();
    assert!(g.stats.is_empty() && !g.streams.is_empty());
    florilegium_core::reset();
    let mut f = args();
    f.frame = Some(glue::Frame { k: 8, n: 8, explain: false });
    let last = glue::garden(&f).unwrap();
    florilegium_core::reset();
    let page = glue::garden(&args()).unwrap();
    // the last frame is exactly the page (tools/animcheck.lua)
    assert_eq!(last.streams, page.streams);
}

#[test]
fn initial_draws_and_reports_a_missing_glyph() {
    let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../fonts/lmroman10-bold.otf")).unwrap();
    let shapes = initial::shapes_from_font(&bytes).unwrap();
    let side = svg::sp("2cm").unwrap();
    let svg = glue::initial("A", side, svg::sp("6mm").unwrap(), svg::sp("7pt").unwrap(), &shapes, 3).unwrap();
    assert!(svg.contains("<path"), "{}", &svg[..svg.len().min(200)]);
    let err = glue::initial("\u{4e2d}", side, 0, side, &shapes, 3).unwrap_err();
    assert!(err.starts_with("florilegium: "), "{err}");
    // the state is clean again
    assert_eq!(glue::initial("A", side, svg::sp("6mm").unwrap(), svg::sp("7pt").unwrap(), &shapes, 3).unwrap(), svg);
}

/// Typst's length conversion, round(bp * 72.27/72 * 65536), against TeX's scan_dimen
#[test]
fn typst_lengths_match_tex() {
    let typst = |bp: f64| (bp * 72.27 / 72.0 * 65536.0).round() as i64;
    let mm = |x: f64| x * (72.0 / 25.4);
    let cases: [(&str, f64, &str); 9] = [
        ("68mm", mm(68.0), "68mm"), ("22mm", mm(22.0), "22mm"), ("32mm", mm(32.0), "32mm"),
        ("30mm", mm(30.0), "30mm"), ("8mm", mm(8.0), "8mm"), ("12pt", 12.0, "12bp"), ("30pt", 30.0, "30bp"),
        ("a4", mm(210.0), "210mm"), ("letter", 8.5 * 72.0, "8.5in"),
    ];
    let mut report = vec![];
    for (name, bp, tex) in cases {
        let (t, l) = (typst(bp), svg::sp(tex).unwrap());
        report.push(format!("{name}: typst {t} sp, scan_dimen {l} sp, diff {}", t - l));
        assert!((t - l).abs() <= 1, "{}", report.join("\n"));
    }
    // the heights of the two papers too
    assert!((typst(mm(297.0)) - svg::sp("297mm").unwrap()).abs() <= 1);
    assert!((typst(11.0 * 72.0) - svg::sp("11in").unwrap()).abs() <= 1);
    println!("{}", report.join("\n"));
}

#[test]
fn draw_is_stateless() {
    let o = Opts::default();
    florilegium_core::reset();
    let a = glue::draw("flower", 1.2, &o, "", "", None, 5).unwrap();
    // other calls in between, with colors and a seed of their own, change nothing
    glue::seed(3, Some(9));
    glue::draw("lily", 1.0, &o, "5B2A1F", "2F6B3A", None, 6).unwrap();
    glue::draw_ghost("flower", 1.0, &Opts { lift: Some(0.4), ..Default::default() }, "", "", None, 7).unwrap();
    assert_eq!(glue::draw("flower", 1.2, &o, "", "", None, 5).unwrap(), a);
    assert_ne!(glue::draw("flower", 1.2, &o, "", "", None, 6).unwrap(), a);
    florilegium_core::reset();
    assert_eq!(glue::draw("flower", 1.2, &o, "", "", None, 5).unwrap(), a);
}

#[test]
fn huge_seeds_wrap() {
    let _ = glue::seed(i64::MAX, Some(i64::MAX));
    let _ = glue::seed(i64::MAX, None);
    assert!(glue::ornament("flower", 786432, "", "", 0.0, Some(i64::MAX), i64::MAX, None).is_ok());
    assert!(glue::ornament("flower", 786432, "", "", 0.0, None, i64::MAX, None).is_ok());
    let mut a = args();
    a.seed = Some(i64::MAX);
    a.page = i64::MAX;
    assert!(glue::garden(&a).is_ok());
}
