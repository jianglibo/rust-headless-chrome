use std::sync::atomic::{AtomicUsize, Ordering};

static METHOD_ID_ABOVE_10000: AtomicUsize = AtomicUsize::new(10001);

pub fn create_if_no_manual_input(manual: Option<usize>) -> (usize, bool) {
    manual.map_or_else(||(METHOD_ID_ABOVE_10000.fetch_add(1, Ordering::SeqCst), false),|mid|(mid, true))
}

pub fn create_one() -> usize {
    METHOD_ID_ABOVE_10000.fetch_add(1, Ordering::SeqCst)
}