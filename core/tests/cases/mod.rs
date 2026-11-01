// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! The Rust twin of tools/oracle/cases.lua: one module per family.
mod fmt;

pub fn run_case(id: &str) -> Result<String, String> {
    match id.split('/').next().unwrap_or("") {
        "fmt" => fmt::run(id),
        _ => Err(format!("unknown case family in id {id:?}")),
    }
}
