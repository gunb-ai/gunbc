// ABI bridge for std.algebra, the one home of FreeMonoid and its list operations
// (dag/std/algebra.dag). Emitted modules under the self-host behavioral instruments
// import list_snoc_item and FreeMonoid from std.algebra, so both live here, once.
use im::Vector as Vec;
use std::rc::Rc;

pub enum FreeMonoid<T> {
    Empty,
    Cons {
        head: T,
        tail: std::rc::Rc<FreeMonoid<T>>,
    },
}

pub fn list_snoc_item<T: Clone>(xs: Rc<Vec<T>>, item: T) -> Rc<Vec<T>> {
    let mut out = xs.as_ref().clone();
    out.push_back(item);
    Rc::new(out)
}
