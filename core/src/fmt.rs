// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! C printf-exact number formatting (glibc), and Lua 5.4's `tostring`.
use std::fmt::Write;

/// C `%.{prec}f`
pub fn fixed(out: &mut String, x: f64, prec: usize) {
    if !x.is_finite() {
        non_finite(out, x);
        return;
    }
    if prec <= 15 {
        let y = x * 10f64.powi(prec as i32);
        let ay = y.abs();
        if ay < 4503599627370496.0 && (ay - ay.floor() - 0.5).abs() > 1e-6 {
            let digits = ay.round() as u64;
            if x.is_sign_negative() {
                out.push('-');
            }
            let s = digits.to_string();
            if prec == 0 {
                out.push_str(&s);
            } else {
                let padded = format!("{s:0>width$}", width = prec + 1);
                let (int, frac) = padded.split_at(padded.len() - prec);
                let _ = write!(out, "{int}.{frac}");
            }
            return;
        }
    }
    let _ = write!(out, "{:.*}", prec, x);
}

fn non_finite(out: &mut String, x: f64) {
    if x.is_nan() {
        out.push_str(if x.is_sign_negative() { "-nan" } else { "nan" });
    } else {
        out.push_str(if x < 0.0 { "-inf" } else { "inf" });
    }
}

/// C `%g`
pub fn g(out: &mut String, x: f64) {
    gp(out, x, 6);
}

/// C `%.{prec}g`
pub fn gp(out: &mut String, x: f64, prec: usize) {
    if !x.is_finite() {
        non_finite(out, x);
        return;
    }
    let p = prec.max(1);
    if x == 0.0 {
        out.push_str(if x.is_sign_negative() { "-0" } else { "0" });
        return;
    }
    let e = format!("{:.*e}", p - 1, x);
    let (mant, exp) = e.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    if exp >= -4 && exp < p as i32 {
        let s = format!("{:.*}", (p as i32 - 1 - exp) as usize, x);
        out.push_str(strip_zeros(&s));
    } else {
        out.push_str(strip_zeros(mant));
        let _ = write!(out, "e{}{:02}", if exp < 0 { '-' } else { '+' }, exp.abs());
    }
}

fn strip_zeros(s: &str) -> &str {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.')
    } else {
        s
    }
}

/// Lua 5.4 `tostring` of a float: `%.14g`, plus `.0` when the result looks like an integer.
pub fn lua_num(out: &mut String, x: f64) {
    let start = out.len();
    gp(out, x, 14);
    if out[start..].bytes().all(|c| c == b'-' || c.is_ascii_digit()) {
        out.push_str(".0");
    }
}

pub fn lua_int(out: &mut String, i: i64) {
    let _ = write!(out, "{i}");
}

#[cfg(test)]
mod tests {
    use super::*;
    fn f(x: f64, p: usize) -> String { let mut s = String::new(); fixed(&mut s, x, p); s }
    fn gg(x: f64) -> String { let mut s = String::new(); g(&mut s, x); s }
    fn ln(x: f64) -> String { let mut s = String::new(); lua_num(&mut s, x); s }

    #[test]
    fn fixed_cases() {
        assert_eq!(f(0.25, 1), "0.2");
        assert_eq!(f(0.35, 1), "0.3");
        assert_eq!(f(0.45, 1), "0.5");
        assert_eq!(f(-0.04, 1), "-0.0");
        assert_eq!(f(1e21, 1), "1000000000000000000000.0");
        assert_eq!(f(2.675, 2), "2.67");
    }
    #[test]
    fn g_cases() {
        assert_eq!(gg(0.5), "0.5");
        assert_eq!(gg(1e-5), "1e-05");
        assert_eq!(gg(123456789.0), "1.23457e+08");
        assert_eq!(gg(0.0001), "0.0001");
        assert_eq!(gg(100000.0), "100000");
        assert_eq!(gg(1e6), "1e+06");
    }
    #[test]
    fn lua_num_cases() {
        assert_eq!(ln(3.0), "3.0");
        assert_eq!(ln(0.1), "0.1");
        assert_eq!(ln(1e15), "1e+15");
        assert_eq!(ln(2.0f64.powf(53.0)), "9.007199254741e+15");
        assert_eq!(ln(f64::INFINITY), "inf");
        assert_eq!(ln(-f64::INFINITY), "-inf");
        assert_eq!(ln(-0.0), "-0.0");
    }
}
