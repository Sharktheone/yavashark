mod env;
mod intrinsics;

use crate::global::init_global_obj;
use crate::realm::env::Environment;
use crate::realm::intrinsics::Intrinsics;
use crate::scope::Scope;
use crate::{NativeFunction, Object, ObjectHandle, Res, Result, Value, ValueResult};
use std::fmt::Debug;
use std::path::PathBuf;
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

    pub fn set_eval(&mut self, eval: impl Eval + 'static) -> Res {
        let eval_func = NativeFunction::new(
            "eval",
            move |args, _, realm| {
                let Some(code) = args.first() else {
                    return Ok(Value::Undefined);
                };

                let code = code.to_string(realm)?;

                let mut scope = Scope::global(realm, PathBuf::from("eval")); //TODO: the scope should be the caller's scope

                eval.eval(&code, realm, &mut scope)
            },
            self,
        );

        self.intrinsics.eval = Some(eval_func.clone());
        self.global.define_property("eval".into(), eval_func.into())
    }
}

pub trait Eval {
    fn eval(&self, code: &str, realm: &mut Realm, scope: &mut Scope) -> ValueResult;
}

impl Eq for Realm {}

impl RealmT for Realm {}
