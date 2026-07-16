use std::rc::Rc;

use im_rc::Vector as Vec;

use crate::std_algebra::FreeMonoid;
use crate::std_types::Char;

pub fn host_string_text_from_rust_host(host: String) -> Rc<FreeMonoid<Char>> {
    let chars: Vec<Char> = host.chars().map(|c| c as i64).collect();
    if chars.is_empty() {
        return Rc::new(FreeMonoid::Empty);
    }
    let head = chars[0];
    let tail = Rc::new(chars.iter().skip(1).cloned().collect());
    Rc::new(FreeMonoid::Cons { head, tail })
}

pub fn host_string_text_to_rust_host(text: Rc<FreeMonoid<Char>>) -> String {
    fn push_char(acc: &mut String, cp: Char) {
        if let Some(ch) = char::from_u32(cp as u32) {
            acc.push(ch);
        }
    }
    match text.as_ref() {
        FreeMonoid::Empty => String::new(),
        FreeMonoid::Cons { head, tail } => {
            let mut out = String::new();
            push_char(&mut out, *head);
            for cp in tail.iter() {
                push_char(&mut out, *cp);
            }
            out
        }
    }
}
