use std::rc::Rc;

pub type Optional<T> = Option<T>;

pub fn optional_absent<T>() -> Option<T> {
    None
}

pub fn optional_present<T>(value: T) -> Option<T> {
    Some(value)
}
