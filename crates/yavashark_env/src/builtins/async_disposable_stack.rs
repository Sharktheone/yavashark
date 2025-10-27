use yavashark_macro::{object, props};
use crate::Realm;

#[object]
#[derive(Debug)]
pub struct AsyncDisposableStack {}


#[props(intrinsic_name = async_disposable_stack)]
impl AsyncDisposableStack {
}