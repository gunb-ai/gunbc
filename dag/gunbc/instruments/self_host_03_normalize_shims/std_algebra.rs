// Minimal ABI stub: normalize imports FreeMonoid only.
pub enum FreeMonoid<T> {
    Empty,
    Cons {
        head: T,
        tail: std::rc::Rc<FreeMonoid<T>>,
    },
}
