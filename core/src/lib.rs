// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
pub mod figures;
pub mod flora;
pub mod fmt;
pub mod grow;
pub mod ink;

/// Puts every piece of thread-local state back to what a fresh Lua process has.
pub fn reset() {
    ink::reset();
    grow::reset();
}
