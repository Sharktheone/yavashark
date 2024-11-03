mod env;
mod intrinsics;

use crate::global::init_global_obj;
use crate::realm::env::Environment;
use crate::realm::intrinsics::Intrinsics;
use crate::{Object, ObjectHandle, Result};
use anyhow::anyhow;
use std::fmt::Debug;
use yavashark_value::Realm as RealmT;

#[derive(Debug, Clone, PartialEq)]
pub struct Realm {
    pub intrinsics: Intrinsics, // [[Intrinsics]]
    pub global: ObjectHandle,   // [[GlobalObject]]
    pub env: Environment,       // [[GlobalEnv]]
}

impl Realm {
    pub fn new() -> Result<Self> {
        let intrinsics = Intrinsics::new()?;

        let global = Object::with_proto(intrinsics.obj.clone().into());

        let realm = Self {
            env: Environment {},
            intrinsics,
            global: global.clone(),
        };

        init_global_obj(&global, &realm)?;

        Ok(realm)
    }
}

impl Eq for Realm {}

impl RealmT for Realm {}
