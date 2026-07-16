// Optional import path for dead `use Optional::{Absent, Present}` in emitted entry.
pub mod Optional {
    pub enum Enum<T> {
        Absent,
        Present { value: T },
    }
    pub use Enum::{Absent, Present};
}

pub fn optional_absent<T>() -> Option<T> {
    None
}

pub fn optional_present<T>(value: T) -> Option<T> {
    Some(value)
}
