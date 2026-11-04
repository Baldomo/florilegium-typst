// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! The command line of florilegium-svg.lua: `florilegium-svg --seed 7 > margin.svg` (for debugging).
use florilegium_core::svg;

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let cli = match svg::options(&argv) {
        Ok(c) => c,
        Err(e) => { eprintln!("florilegium-svg: {e}"); std::process::exit(2) }
    };
    if cli.help {
        print!("{}", svg::USAGE);
        return;
    }
    let out = match svg::page(&cli.page) {
        Ok(o) => o,
        Err(e) => { eprintln!("florilegium-svg: {e}"); std::process::exit(1) }
    };
    if !out.captions.is_empty() {
        eprintln!("florilegium-svg: {} equations and symbols need TeX and are left out (the Typst package sets them)", out.captions.len());
    }
    match cli.out {
        Some(f) => std::fs::write(f, out.svg).expect("cannot write the output file"),
        None => print!("{}", out.svg),
    }
}
