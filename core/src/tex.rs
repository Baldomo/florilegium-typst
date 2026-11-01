// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
//! Caption TeX to Typst markup (the Typst layer typesets captions natively).
//!
//! Only the TeX the library itself produces is understood; anything else is an `Err`.
//! In math every token is separated by a space (Typst would read `ab` as one identifier
//! and `f(x)` as a call), except around delimiters, commas and `!`.

#[derive(Clone, Copy, PartialEq)]
enum Kind { Open, Close, Punct, Other }

struct Atom { s: String, kind: Kind, plain: bool }

fn atom(s: &str, kind: Kind) -> Atom { Atom { s: s.to_string(), kind, plain: true } }

fn join(atoms: &[Atom]) -> String {
    let mut out = String::new();
    for (i, a) in atoms.iter().enumerate() {
        if i > 0 && !matches!(a.kind, Kind::Close | Kind::Punct) && atoms[i - 1].kind != Kind::Open { out.push(' ') }
        out.push_str(&a.s);
    }
    out
}

fn command(name: &str) -> Option<&'static str> {
    Some(match name {
        "pi" => "pi", "varphi" => "phi", "lambda" => "lambda", "theta" => "theta", "alpha" => "alpha", "delta" => "delta",
        "zeta" => "zeta", "chi" => "chi", "Omega" => "Omega", "Delta" => "Delta", "Sigma" => "Sigma",
        "partial" => "partial", "nabla" => "nabla", "infty" => "infinity", "aleph" => "aleph",
        "times" => "times", "pm" => "plus.minus", "approx" => "approx", "in" => "in",
        "le" => "<=", "ge" => ">=", "to" => "->", "mapsto" => "|->", "cdots" => "dots.c", "bmod" => "mod",
        "int" => "integral", "sum" => "sum", "prod" => "product", "oint" => "integral.cont",
        "cos" => "cos", "sin" => "sin", "log" => "log", "lim" => "lim", "dim" => "dim",
        "circ" => "degree",
        _ => return None,
    })
}

struct P<'a> { c: Vec<char>, i: usize, src: &'a str }

impl P<'_> {
    fn skip(&mut self) { while self.c.get(self.i) == Some(&' ') { self.i += 1 } }
    fn err<T>(&self, what: &str) -> Result<T, String> { Err(format!("{what} in TeX caption {:?}", self.src)) }

    /// atoms up to the end, an unmatched `}` (consumed when `brace`), `\bigr`, or a closing `|`
    fn seq(&mut self, brace: bool, bar: bool) -> Result<Vec<Atom>, String> {
        let mut out = vec![];
        loop {
            self.skip();
            match self.c.get(self.i) {
                None => return if brace || bar { self.err("unterminated group") } else { Ok(out) },
                Some('}') if brace => { self.i += 1; return Ok(out) }
                Some('|') if bar => { self.i += 1; return Ok(out) }
                Some('\\') if !brace && !bar && self.c[self.i..].starts_with(&['\\', 'b', 'i', 'g', 'r']) => return Ok(out),
                Some('\\') if brace && self.c[self.i..].starts_with(&['\\', 'b', 'i', 'g', 'r']) => return self.err("stray \\bigr"),
                Some(_) => out.push(self.scripted(false)?),
            }
        }
    }

    /// an argument: a braced group, or a single token
    fn arg(&mut self) -> Result<Atom, String> {
        self.skip();
        if self.c.get(self.i) == Some(&'{') {
            self.i += 1;
            let g = self.seq(true, false)?;
            if g.len() == 1 && g[0].plain { return Ok(g.into_iter().next().unwrap()) }
            if g.is_empty() { return self.err("empty group") }
            return Ok(Atom { s: join(&g), kind: Kind::Other, plain: false });
        }
        if self.i >= self.c.len() { return self.err("missing argument") }
        self.atom(true)
    }

    fn scripted(&mut self, single: bool) -> Result<Atom, String> {
        let mut a = self.atom(single)?;
        loop {
            self.skip();
            let op = match self.c.get(self.i) { Some(&c @ ('^' | '_')) => c, _ => return Ok(a) };
            self.i += 1;
            let g = self.arg()?;
            a.s.push(op);
            if g.plain { a.s.push_str(&g.s) } else { a.s.push('('); a.s.push_str(&g.s); a.s.push(')') }
            a.plain = false;
            a.kind = Kind::Other;
        }
    }

    fn call(&mut self, f: &str, n: usize) -> Result<Atom, String> {
        let mut args = vec![];
        for _ in 0..n { args.push(self.arg()?.s) }
        Ok(atom(&format!("{f}({})", args.join(", ")), Kind::Other))
    }

    /// one token; `single` takes one character of a number (TeX's `^12` is `^1` and `2`)
    fn atom(&mut self, single: bool) -> Result<Atom, String> {
        self.skip();
        let Some(&c) = self.c.get(self.i) else { return self.err("unexpected end") };
        self.i += 1;
        Ok(match c {
            '0'..='9' => {
                let from = self.i - 1;
                while !single && matches!(self.c.get(self.i), Some('0'..='9' | '.')) { self.i += 1 }
                atom(&self.c[from..self.i].iter().collect::<String>(), Kind::Other)
            }
            'a'..='z' | 'A'..='Z' => atom(&c.to_string(), Kind::Other),
            '!' => atom("!", Kind::Punct),
            '+' | '-' | '=' | '<' | '>' => atom(&c.to_string(), Kind::Other),
            ',' | ':' => atom(&c.to_string(), Kind::Punct),
            '(' | '[' => atom(&c.to_string(), Kind::Open),
            ')' | ']' => atom(&c.to_string(), Kind::Close),
            '{' => {
                let g = self.seq(true, false)?;
                Atom { s: join(&g), kind: Kind::Other, plain: false }
            }
            '|' => {
                let g = self.seq(false, true)?;
                Atom { s: format!("abs({})", join(&g)), kind: Kind::Other, plain: true }
            }
            '\\' => {
                let Some(&d) = self.c.get(self.i) else { return self.err("trailing backslash") };
                if !d.is_ascii_alphabetic() {
                    self.i += 1;
                    return Ok(match d {
                        ',' => atom("thin", Kind::Other),
                        ' ' | ';' => atom("space", Kind::Other),
                        '{' => atom("\\{", Kind::Open),
                        '}' => atom("\\}", Kind::Close),
                        _ => return self.err(&format!("unsupported \\{d}")),
                    });
                }
                let from = self.i;
                while self.c.get(self.i).map_or(false, |x| x.is_ascii_alphabetic()) { self.i += 1 }
                let name: String = self.c[from..self.i].iter().collect();
                match name.as_str() {
                    "frac" => self.call("frac", 2)?,
                    "sqrt" => self.call("sqrt", 1)?,
                    "mathrm" => self.call("upright", 1)?,
                    "mathcal" => self.call("cal", 1)?,
                    "displaystyle" => {
                        let a = self.atom(false)?;
                        atom(&format!("display({})", a.s), Kind::Other)
                    }
                    "bigl" => {
                        if self.c.get(self.i) != Some(&'(') { return self.err("\\bigl without (") }
                        self.i += 1;
                        let g = self.seq(false, false)?;
                        if !self.c[self.i..].starts_with(&['\\', 'b', 'i', 'g', 'r', ')']) { return self.err("\\bigl without \\bigr)") }
                        self.i += 6;
                        Atom { s: format!("lr(( {} ))", join(&g)), kind: Kind::Other, plain: true }
                    }
                    n => match command(n) {
                        Some(t) => atom(t, Kind::Other),
                        None => return self.err(&format!("unsupported \\{n}")),
                    },
                }
            }
            _ => return self.err(&format!("unsupported character {c:?}")),
        })
    }
}

/// TeX math (without the dollar signs) to Typst math
pub fn math_to_typst(tex: &str) -> Result<String, String> {
    let mut p = P { c: tex.chars().collect(), i: 0, src: tex };
    let atoms = p.seq(false, false)?;
    if p.i < p.c.len() { return p.err("stray \\bigr") }
    Ok(join(&atoms))
}

/// Text with `$…$` math to Typst markup
pub fn to_typst(tex: &str) -> Result<String, String> {
    let parts: Vec<&str> = tex.split('$').collect();
    if parts.len() % 2 == 0 { return Err(format!("unbalanced $ in TeX caption {tex:?}")) }
    let mut out = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i % 2 == 1 {
            out.push('$');
            out.push_str(&math_to_typst(part)?);
            out.push('$');
        } else {
            for c in part.chars() {
                if "\\*_#[]@<`~".contains(c) { out.push('\\') }
                out.push(c);
            }
        }
    }
    Ok(out)
}
