// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
use florilegium_core::figures::{self, FAMILY_ORDER, FIGURE_ORDER};
use florilegium_core::ink::Rng;
use florilegium_core::math::{EQUATIONS, GLYPHS};
use florilegium_core::tex::{math_to_typst, to_typst};
use std::collections::BTreeSet;

const GLYPH_TYPST: [&str; 53] = [
    "upright(A)", "upright(M)", "upright(Q)", "upright(B)", "upright(D)", "cal(F)", "cal(L)", "cal(R)",
    "x", "y", "n", "k", "e", "i", "z",
    "0", "1", "2", "3", "4", "5", "7", "8", "9",
    "+", "=", "-", "times", "plus.minus", "(", ")", "[", "]", "!", "approx", "in", "<=",
    "pi", "phi", "lambda", "theta", "Omega", "Delta", "Sigma", "partial", "nabla", "infinity", "aleph",
    "display(integral)", "display(sum)", "display(product)", "display(integral.cont)", "sqrt(space)",
];

const EQUATION_TYPST: [&str; 23] = [
    "e^(i pi) + 1 = 0",
    "a^2 + b^2 = c^2",
    "phi^2 = phi + 1",
    "V - E + F = 2",
    "i^2 = - 1",
    "sum_(n >= 1) frac(1, n^2) = frac(pi^2, 6)",
    "integral_(- infinity)^infinity e^(- x^2) d x = sqrt(pi)",
    "F_(n + 1) = F_n + F_(n - 1)",
    "cos^2 theta + sin^2 theta = 1",
    "x = frac(- b plus.minus sqrt(b^2 - 4 a c), 2 a)",
    "e^x = sum_(n >= 0) frac(x^n, n!)",
    "e = sum_(n >= 0) frac(1, n!)",
    "frac(d, d x) e^x = e^x",
    "1 + 2 + dots.c + n = frac(n (n + 1), 2)",
    "zeta (s) = product_p frac(1, 1 - p^(- s))",
    "z |-> z^2 + c",
    "lim_(n -> infinity) lr(( 1 + frac(1, n) ))^n = e",
    "pi approx frac(355, 113)",
    "dim = log_3 4",
    "r = cos k theta",
    "integral.cont_C f thin d z = 0",
    "2^(aleph_0) > aleph_0",
    "n sin frac(pi, n) -> pi",
];

#[test]
fn fixed_glyphs_and_equations() {
    for (t, want) in GLYPHS.iter().zip(GLYPH_TYPST) {
        assert_eq!(math_to_typst(t).as_deref(), Ok(want), "glyph {t}");
        assert_eq!(to_typst(&format!("${t}$")), Ok(format!("${want}$")), "glyph {t}");
    }
    for (t, want) in EQUATIONS.iter().zip(EQUATION_TYPST) {
        assert_eq!(math_to_typst(t).as_deref(), Ok(want), "equation {t}");
    }
}

#[test]
fn text_and_errors() {
    assert_eq!(to_typst("Vogel's disc: floret $n$ at $n\\alpha$").unwrap(), "Vogel's disc: floret $n$ at $n alpha$");
    assert_eq!(to_typst("a*b_c#d[e]$f$@g<h`i~").unwrap(), "a\\*b\\_c\\#d\\[e\\]$f$\\@g\\<h\\`i\\~");
    assert_eq!(to_typst("$T_{2,9}$").unwrap(), "$T_(2, 9)$");
    assert_eq!(to_typst("tetrahedron $\\{3,3\\}$").unwrap(), "tetrahedron $\\{3, 3\\}$");
    assert_eq!(to_typst("$\\alpha=137.5^\\circ$").unwrap(), "$alpha = 137.5^degree$");
    assert_eq!(to_typst("$|x|^n+|y|^n=1$").unwrap(), "$abs(x)^n + abs(y)^n = 1$");
    assert_eq!(to_typst("$k\\mapsto mk \\bmod 100$").unwrap(), "$k |-> m k mod 100$");
    assert_eq!(to_typst("$x=\\sin 3t,\\ y=\\sin 2t$").unwrap(), "$x = sin 3 t, space y = sin 2 t$");
    assert_eq!(to_typst("$d=\\frac14r$").unwrap(), "$d = frac(1, 4) r$");
    assert!(to_typst("$\\foo$").is_err());
    assert!(to_typst("$x").is_err());
    assert!(to_typst("$\\frac{1}$").is_err());
}

fn all_caption_strings() -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for t in GLYPHS { set.insert(format!("${t}$")); }
    for t in EQUATIONS { set.insert(format!("${t}$")); }
    for name in FIGURE_ORDER {
        let f = figures::figure(name).unwrap();
        for seed in 1..=300 {
            let r = Rng::new(seed as f64);
            for _ in 0..20 { set.insert((f.caption)(&(f.pick)(&r))); }
        }
    }
    for name in FAMILY_ORDER {
        let f = figures::family(name).unwrap();
        for seed in 1..=300 {
            let r = Rng::new(seed as f64);
            for _ in 0..20 { set.insert((f.title)(&(f.pick)(&r))); }
        }
        for v in &f.values { set.insert((f.label)(v)); }
    }
    set
}

const LIST: &str = "tests/data/captions.txt";

#[test]
fn committed_list_is_current() {
    let now: Vec<String> = all_caption_strings().into_iter().collect();
    let text = now.join("\n") + "\n";
    if std::env::var("FLORILEGIUM_REGEN").is_ok() {
        std::fs::create_dir_all("tests/data").unwrap();
        std::fs::write(LIST, &text).unwrap();
    }
    let have = std::fs::read_to_string(LIST).expect("tests/data/captions.txt (FLORILEGIUM_REGEN=1 to write)");
    assert_eq!(have, text, "captions.txt is stale; rerun with FLORILEGIUM_REGEN=1");
}

fn converted() -> Vec<(String, String)> {
    let list = std::fs::read_to_string(LIST).unwrap();
    list.lines().map(|l| (l.to_string(), to_typst(l).unwrap_or_else(|e| panic!("{l}: {e}")))).collect()
}

#[test]
fn every_caption_converts() {
    assert!(converted().len() > 150);
}

fn typst_cmd() -> Option<std::process::Command> {
    let mise = std::path::PathBuf::from(std::env::var("HOME").ok()?).join(".local/bin/mise");
    let mut c = std::process::Command::new(mise);
    c.args(["exec", "--", "typst"]).current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."));
    let out = c.arg("--version").output().ok()?;
    String::from_utf8_lossy(&out.stdout).contains("0.15.1").then(|| {
        let mut c = std::process::Command::new(std::env::var("HOME").unwrap() + "/.local/bin/mise");
        c.args(["exec", "--", "typst"]).current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."));
        c
    })
}

#[test]
fn every_caption_compiles() {
    let Some(mut cmd) = typst_cmd() else { println!("typst 0.15.1 unreachable; skipping compile check"); return };
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/captions");
    std::fs::create_dir_all(&dir).unwrap();
    let mut typ = String::from("#set page(width: 16cm, height: auto, margin: 0.5cm)\n#set text(size: 11pt)\n");
    for (i, (tex, m)) in converted().iter().enumerate() {
        let lit = m.replace('\\', "\\\\").replace('"', "\\\"");
        typ.push_str(&format!("#box(width: 100%)[#raw(\"{}\") \\ #eval(\"{lit}\", mode: \"markup\") #h(1fr) #text(size: 6pt)[{i}]]\n\n", tex.replace('\\', "\\\\").replace('"', "\\\"")));
    }
    let src = dir.join("captions.typ");
    std::fs::write(&src, typ).unwrap();
    let out = cmd.args(["compile", "--format", "png", "--ppi", "110", src.to_str().unwrap(), dir.join("captions{p}.png").to_str().unwrap()]).output().unwrap();
    assert!(out.status.success(), "typst failed:\n{}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn list_covers_the_lua_run() {
    let lua = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../tools/oracle/case.lua");
    let mise = std::env::var("HOME").unwrap() + "/.local/bin/mise";
    let out = std::process::Command::new(mise).args(["exec", "--", "lua"]).arg(&lua).arg("figures/captions")
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/..")).output();
    let Ok(out) = out else { println!("lua unreachable; skipping"); return };
    if !out.status.success() { println!("lua failed; skipping: {}", String::from_utf8_lossy(&out.stderr)); return }
    let have: BTreeSet<String> = std::fs::read_to_string(LIST).unwrap().lines().map(String::from).collect();
    let mut n = 0;
    for l in String::from_utf8_lossy(&out.stdout).lines() {
        let Some((_, t)) = l.split_once('\t') else { continue };
        n += 1;
        assert!(have.contains(t), "Lua caption missing from the list: {t}");
    }
    assert!(n > 100, "only {n} Lua captions read");
}
