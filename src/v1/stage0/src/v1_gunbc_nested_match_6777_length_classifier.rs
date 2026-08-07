// Snapshot of main's broken FreeMonoid native-match lowering for #6777 reproducer.
// Sibling Cons arms collapse to the first arm body (One) for every nonempty list.
// Replaced by correct nested-match grouping when the construction fix lands.
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LengthClass {
    Zero,
    One,
    Many,
}

pub fn mk_list(n: i64) -> Rc<Vec<i64>> {
    if n <= 0 {
        Rc::new(vec![])
    } else {
        let mut __cons_v = (*mk_list(n - 1)).clone();
        __cons_v.insert(0, 1);
        Rc::new(__cons_v)
    }
}

pub fn classify_length(xs: Rc<Vec<i64>>) -> LengthClass {
    let __fm = xs.clone();
    if __fm.is_empty() {
        LengthClass::Zero
    } else {
        LengthClass::One
    }
}
