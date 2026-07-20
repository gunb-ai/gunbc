use im::Vector as Vec;
use std::sync::Arc;

pub fn list_snoc_item<T: Clone>(xs: Arc<Vec<T>>, item: T) -> Arc<Vec<T>> {
    let mut out = xs.as_ref().clone();
    out.push_back(item);
    Arc::new(out)
}
