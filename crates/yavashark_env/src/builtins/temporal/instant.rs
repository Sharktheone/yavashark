use std::cell::Cell;
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct Instant {
    dur: Cell<std::time::Instant>,
    negative: Cell<bool>,
}

#[props]
impl Instant {}
