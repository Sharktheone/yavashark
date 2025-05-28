mod env;
mod intrinsics;
mod resolve;

use crate::global::init_global_obj;
use crate::realm::env::Environment;
use crate::realm::intrinsics::Intrinsics;
use crate::scope::Scope;
use crate::task_queue::AsyncTaskQueue;
use crate::{NativeFunction, Object, ObjectHandle, Res, Value, ValueResult, Variable};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::PathBuf;
use yavashark_value::Realm as RealmT;

#[derive(Debug, Clone, PartialEq)]
pub struct Realm {
    pub intrinsics: Intrinsics, // [[Intrinsics]]
    pub global: ObjectHandle,   // [[GlobalObject]]
    pub env: Environment,       // [[GlobalEnv]]
    pub queue: AsyncTaskQueue,
}

impl Realm {
    pub fn new() -> Res<Self> {
        let intrinsics = Intrinsics::new()?;

        let global = Object::with_proto(intrinsics.obj.clone().into());

        let mut realm = Self {
            env: Environment {
                modules: HashMap::new(),
            },
            intrinsics,
            global: global.clone(),
            queue: AsyncTaskQueue::new(),
        };

        init_global_obj(&global, &mut realm)?;

        Ok(realm)
    }

    pub fn set_eval(&mut self, eval: impl Eval + 'static) -> Res {
        let eval_func = NativeFunction::with_len(
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
            1,
        );

        self.intrinsics.eval = Some(eval_func.clone());
        self.global
            .define_variable("eval".into(), Variable::write_config(eval_func.into()))
    }

    pub async fn run_event_loop(&mut self) {
        self.queue.runner().run(self).await;
    }
}

pub trait Eval {
    fn eval(&self, code: &str, realm: &mut Realm, scope: &mut Scope) -> ValueResult;
}

impl Eq for Realm {}

impl RealmT for Realm {}
