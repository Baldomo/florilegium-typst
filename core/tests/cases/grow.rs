// Copyright (c) 2026 Kevin Raymond (florilegium); Typst port. MIT License.
use florilegium_core::fmt;
use florilegium_core::grow::{self, Mark, MarkKind, G};
use florilegium_core::ink::{self, Event, Tag};
use std::cell::RefCell;
use std::rc::Rc;

fn marks() -> Vec<Mark> {
    vec![
        Mark { kind: MarkKind::Heading(1), y: 20.0 },
        Mark { kind: MarkKind::Heading(2), y: 14.0 },
        Mark { kind: MarkKind::Bloom(String::new()), y: 9.0 },
        Mark { kind: MarkKind::Bloom("umbel".into()), y: 6.0 },
    ]
}

fn geometry(seed: f64, spill: i32, case: &str) -> Result<G, String> {
    let mut g = G { pw: 21.0, ph: 29.7, textleft: 6.8, textright: 18.8, ytop: 25.4, ybot: 3.0, ..Default::default() };
    g.colright = g.textleft - 0.4;
    g.headrule = g.ytop + 0.7;
    g.footy = 1.9;
    g.seed = seed;
    g.spill = spill;
    g.spilly = if spill == 1 { 19.0 } else { 0.0 };
    match case {
        "summer" => {}
        "novis" => g.visitors = Some(false),
        "spring" => g.age = Some(0.05),
        "autumn-marks" => { g.age = Some(0.95); g.marks = marks() }
        "density-0.4" => g.density = Some(0.4),
        "density-0.05" => g.density = Some(0.05),
        "density-1" => g.density = Some(1.0),
        "age-0.5" => g.age = Some(0.5),
        "mirror" => { g.mirror = true; g.marks = marks() }
        _ => return Err(format!("bad grow case {case:?}")),
    }
    Ok(g)
}

fn finish(out: Vec<String>) -> String {
    let mut s = out.join("\n");
    s.push_str("\n=====\n");
    s.push_str(&ink::take_output().join("\n=====\n"));
    s
}

fn frame(rec: &ink::Recording, t: f64, explain: bool) {
    ink::emit(rec, &|tag: &Tag| {
        if tag.overlay && !explain { return false; }
        let e = tag.event.borrow();
        let p = (t - e.t0) / e.dur;
        p >= tag.from && tag.to.map_or(true, |to| p < to)
    });
}

pub fn run(id: &str) -> Result<String, String> {
    let (anim, rest) = match id.strip_prefix("grow/anim/") {
        Some(r) => (true, r),
        None => (false, id.strip_prefix("grow/").ok_or("bad grow id")?),
    };
    let p: Vec<&str> = rest.split('/').collect();
    let bad = || format!("bad grow id {id:?}");
    if p.len() != 3 { return Err(bad()) }
    let seed: f64 = p[0].strip_prefix("seed=").ok_or_else(bad)?.parse().map_err(|_| bad())?;
    let spill: i32 = p[1].strip_prefix("spill=").ok_or_else(bad)?.parse().map_err(|_| bad())?;
    let case = p[2].strip_prefix("case=").ok_or_else(bad)?;
    let mut g = geometry(seed, spill, case)?;
    let mut out = vec![];
    if !anim {
        let r = grow::page(&g)?;
        let st = r.stats.ok_or("no stats")?;
        out.push(format!("motifs {} stems {} cells {}", st.motifs, st.stems, st.cells));
        for (k, v) in &st.counts { out.push(format!("count {k} {v}")) }
        return Ok(finish(out));
    }
    g.anim = true;
    let r = grow::page(&g)?;
    let rec = r.rec.ok_or("no recording")?;
    frame(&rec, f64::INFINITY, false);
    frame(&rec, r.total * 0.5, true);
    let events: RefCell<Vec<Event>> = RefCell::new(vec![]);
    ink::emit(&rec, &|tag: &Tag| {
        let mut ev = events.borrow_mut();
        if !ev.iter().any(|e| Rc::ptr_eq(e, &tag.event)) { ev.push(tag.event.clone()) }
        false
    });
    let events = events.into_inner();
    let mut s = String::from("total ");
    fmt::gp(&mut s, r.total, 17);
    out.push(s);
    out.push(format!("events {}", events.len()));
    for e in &events {
        let e = e.borrow();
        let mut s = String::new();
        fmt::gp(&mut s, e.t0, 17);
        s.push(' ');
        fmt::gp(&mut s, e.dur, 17);
        out.push(s);
    }
    Ok(finish(out))
}
