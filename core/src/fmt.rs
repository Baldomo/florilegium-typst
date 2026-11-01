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
        let y = x * [1.0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15][prec];
        let ay = y.abs();
        if ay < 1073741824.0 && (ay - ay.floor() - 0.5).abs() > 1e-6 {
            let digits = ay.round() as u64;
            if x.is_sign_negative() {
                out.push('-');
            }
            const P10: [u64; 16] = [1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000,
                10000000000, 100000000000, 1000000000000, 10000000000000, 100000000000000, 1000000000000000];
            let p10 = P10[prec];
            push_u64(out, digits / p10);
            if prec > 0 {
                out.push('.');
                let mut f = digits % p10;
                let mut buf = [b'0'; 16];
                for i in (0..prec).rev() {
                    buf[i] = b'0' + (f % 10) as u8;
                    f /= 10;
                }
                push_ascii(out, &buf[..prec]);
            }
            return;
        }
    }
    let _ = write!(out, "{:.*}", prec, x);
}

fn push_u64(out: &mut String, mut n: u64) {
    let mut buf = [0u8; 20];
    let mut i = 20;
    loop {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 { break }
    }
    push_ascii(out, &buf[i..]);
}

fn push_ascii(out: &mut String, b: &[u8]) {
    debug_assert!(b.is_ascii());
    // SAFETY: only ASCII digits are appended
    unsafe { out.as_mut_vec().extend_from_slice(b) }
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
        assert_eq!(f(123456789.04999999, 1), "123456789.0");
        assert_eq!(f(9007199254.04999, 1), "9007199254.0");
        assert_eq!(f(123456789012.35, 1), "123456789012.4");
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
