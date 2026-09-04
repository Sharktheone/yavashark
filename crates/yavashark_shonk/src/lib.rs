
// Testing ground for new YS to improve perf!

// We need to test:
// - new Object API with shapes
// - NaN boxing model
// - Realm and Scope APIs
// - Ability for async Scopes
// - How we handle escaping variables
// - Small micro benches to compare against different engines and current Yavashark

use std::ptr::NonNull;

type Value = u64;


// this will be the global "JS stack" where every stack frame stores it's locals.
pub struct Locals {
    base: Box<[Value]>,
    ptr: NonNull<Value>,
}


// the other option would be to have the JS Locals on the OS Stack with `alloca`
pub struct AllocaLocals<'a> {
    locals: &'a mut [Value],
}

