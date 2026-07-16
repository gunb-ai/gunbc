use std::rc::Rc;

pub fn host_string_text_from_rust_host(host: String) -> Rc<String> {
    Rc::new(host)
}

pub fn host_string_text_to_rust_host(text: Rc<String>) -> String {
    (*text).clone()
}
