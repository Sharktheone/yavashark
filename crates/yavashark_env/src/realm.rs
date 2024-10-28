mod env;
mod intrinsics;

use crate::realm::env::Environment;
use crate::realm::intrinsics::Intrinsics;
use crate::{Object, ObjectHandle};
use std::fmt::Debug;
use yavashark_value::Realm as RealmT;

#[derive(Debug, Clone, PartialEq)]
pub struct Realm {
    pub intrinsics: Intrinsics, // [[Intrinsics]]
    pub global: ObjectHandle,   // [[GlobalObject]]
    pub env: Environment,       // [[GlobalEnv]]
}

impl Realm {
    pub fn new() -> anyhow::Result<Self> {
        let intrinsics = Intrinsics::new()?;

        let global = Object::with_proto(intrinsics.obj.clone().into());

        Ok(Self {
            env: Environment {},
            intrinsics,
            global,
        })
    }
}

impl Eq for Realm {}

impl RealmT for Realm {}