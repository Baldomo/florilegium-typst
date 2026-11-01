// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! The Rust twin of tools/oracle/cases.lua: one module per family.
mod figures;
mod flora;
mod fmt;
mod grow;
mod ink;

pub fn run_case(id: &str) -> Result<String, String> {
    match id.split('/').next().unwrap_or("") {
        "figures" => figures::run(id),
        "flora" => flora::run(id),
        "fmt" => fmt::run(id),
        "grow" => grow::run(id),
        "ink" => ink::run(id),
        _ => Err(format!("unknown case family in id {id:?}")),
    }
}
