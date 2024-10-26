mod intrinsics;
mod env;

use crate::{Object, ObjectHandle};
use crate::realm::env::Environment;
use crate::realm::intrinsics::Intrinsics;

pub struct Realm {
    pub intrinsics: Intrinsics,// [[Intrinsics]]
    pub global: ObjectHandle, // [[GlobalObject]]
    pub env: Environment, // [[GlobalEnv]]
}



impl Realm {
    fn new() -> anyhow::Result<Self> {
        
        
        let intrinsics = Intrinsics::new()?;
        
        let global = Object::with_proto(intrinsics.obj.clone().into());
        
        
        Ok(Self {
            env: Environment {},
            intrinsics,
            global,
        })
    }
}



