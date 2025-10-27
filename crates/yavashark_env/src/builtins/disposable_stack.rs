use yavashark_macro::{object, props};
use crate::Realm;

#[object]
#[derive(Debug)]
pub struct DisposableStack {}


#[props(intrinsic_name = disposable_stack)]
impl DisposableStack {
}