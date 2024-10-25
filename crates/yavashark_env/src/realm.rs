mod intrinsics;
mod env;

use crate::ObjectHandle;
use crate::realm::env::Environment;
use crate::realm::intrinsics::Intrinsics;

pub struct Realm {
    intrinsics: Intrinsics,// [[Intrinsics]]
    global: ObjectHandle, // [[GlobalObject]]
    env: Environment, // [[GlobalEnv]]
}