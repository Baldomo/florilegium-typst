// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
mod cases;

use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::process::Command;

fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")) }

fn escape(id: &str) -> String {
    id.chars().map(|c| if c.is_ascii_alphanumeric() || "-=._".contains(c) { c } else { '_' }).collect()
}

fn lua_side(id: &str) -> Option<String> {
    let script = root().join("../tools/oracle/case.lua");
    let via_mise = Command::new("mise").args(["exec", "--", "lua"]).arg(&script)
        .arg(id).current_dir(root().join("..")).output().ok().filter(|o| o.status.success());
    let out = via_mise.or_else(|| Command::new("lua").arg(&script).arg(id).output().ok())?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

fn first_diff(a: &str, b: &str) -> String {
    let t = |s: &str| s.chars().take(100).collect::<String>();
    let mut la = a.split('\n');
    let mut lb = b.split('\n');
    let mut n = 1;
    loop {
        match (la.next(), lb.next()) {
            (Some(x), Some(y)) if x == y => n += 1,
            (None, None) => return "no differing line".into(),
            (x, y) => return format!("line {n}: rust {:?} / lua {:?}", t(x.unwrap_or("<eof>")), t(y.unwrap_or("<eof>"))),
        }
    }
}

pub fn check_family(prefix: &str) {
    let tsv = std::fs::read_to_string(root().join("tests/fixtures/oracle.tsv")).expect("oracle.tsv");
    let mut failures = vec![];
    let mut count = 0;
    for line in tsv.lines().filter(|l| l.starts_with(prefix)) {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() != 3 { failures.push(format!("{line:?}: malformed fixture line")); continue; }
        let (id, bytes, sha) = (cols[0], cols[1], cols[2]);
        count += 1;
        florilegium_core::reset();
        let got = match cases::run_case(id) {
            Ok(s) => s,
            Err(e) => { failures.push(format!("{id}: {e}")); continue; }
        };
        let got_sha = format!("{:x}", Sha256::digest(got.as_bytes()));
        if got.len().to_string() == bytes && got_sha == sha { continue; }
        let dir = root().join("target/oracle");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join(format!("{}.rust", escape(id))), &got);
        let detail = match lua_side(id) {
            Some(l) => {
                let _ = std::fs::write(dir.join(format!("{}.lua", escape(id))), &l);
                first_diff(&got, &l)
            }
            None => "lua oracle unavailable".into(),
        };
        failures.push(format!("{id}: {} bytes (expected {bytes}); {detail}", got.len()));
    }
    if count == 0 { failures.push(format!("no fixtures with prefix {prefix:?}")); }
    assert!(failures.is_empty(), "{} of {count} cases failed:\n{}", failures.len(), failures.join("\n"));
}

#[test]
fn fmt() { check_family("fmt/"); }

#[test]
fn ink() { check_family("ink/"); }

#[test]
fn flora() { check_family("flora/"); }

#[test]
fn figures() { check_family("figures/"); }

#[test]
fn grow() { check_family("grow/"); }

#[test]
fn authored() { check_family("authored/"); }

#[test]
fn geometric() { check_family("geometric/"); }
