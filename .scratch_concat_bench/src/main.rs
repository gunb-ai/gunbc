use im::Vector;
use std::rc::Rc;
use std::time::Instant;
fn append(a: Rc<Vector<u64>>, b: &Rc<Vector<u64>>) -> Rc<Vector<u64>> { let mut r = a; Rc::make_mut(&mut r).append((**b).clone()); r }
fn push(a: Rc<Vector<u64>>, b: &Rc<Vector<u64>>) -> Rc<Vector<u64>> { let mut r = a; let m = Rc::make_mut(&mut r); for x in b.iter() { m.push_back(*x); } r }
fn run(f: fn(Rc<Vector<u64>>, &Rc<Vector<u64>>) -> Rc<Vector<u64>>, steps: usize, k: usize) -> (f64, usize) {
    let piece: Rc<Vector<u64>> = Rc::new((0..k as u64).collect());
    let mut best = f64::MAX; let mut len = 0;
    for _ in 0..3 { let t = Instant::now(); let mut acc: Rc<Vector<u64>> = Rc::new(Vector::new()); for _ in 0..steps { acc = f(acc, &piece); } let e = t.elapsed().as_secs_f64(); len = acc.len(); if e < best { best = e; } }
    (best, len)
}
#[allow(dead_code)]
fn once(l: usize, r: usize) -> (f64, f64) {
    let left: Rc<Vector<u64>> = Rc::new((0..l as u64).collect());
    let right: Rc<Vector<u64>> = Rc::new((0..r as u64).collect());
    let mut ba = f64::MAX; let mut bp = f64::MAX;
    for _ in 0..5 {
        let t = Instant::now(); let x = append(left.clone(), &right); let e = t.elapsed().as_secs_f64(); assert_eq!(x.len(), l + r); if e < ba { ba = e; }
        let t = Instant::now(); let y = push(left.clone(), &right); let e = t.elapsed().as_secs_f64(); assert_eq!(y.len(), l + r); if e < bp { bp = e; }
    }
    (ba, bp)
}
fn main() {
    println!("GAP k\tsteps\tappend_s\tpush_s\tpush_over_append");
    for k in [3584usize, 4096, 4352, 4608, 4864, 5120] {
        for steps in [400usize, 800, 1600] {
            let (a, _) = run(append, steps, k); let (p, _) = run(push, steps, k);
            println!("GAP {k}\t{steps}\t{a:.6}\t{p:.6}\t{:.2}", p / a);
        }
    }
}
