use im::Vector as Vec;
use std::rc::Rc;

pub fn list_snoc_item<T: Clone>(xs: Rc<Vec<T>>, item: T) -> Rc<Vec<T>> {
    let mut out = xs.as_ref().clone();
    out.push_back(item);
    Rc::new(out)
}
