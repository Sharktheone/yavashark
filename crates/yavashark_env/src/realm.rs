mod env;
mod initialize;
mod intrinsics;

pub mod resolve;

use crate::global::{init_global_obj, new_global_obj};
use crate::realm::env::Environment;
use crate::realm::intrinsics::Intrinsics;
use crate::scope::Scope;
use crate::task_queue::AsyncTaskQueue;
use crate::{NativeFunction, Object, ObjectHandle, Res, Value, ValueResult, Variable};
pub use initialize::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::rc::Rc;

pub struct PrivateRc<T>(Rc<T>);

impl<T> Deref for PrivateRc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for PrivateRc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Rc::get_mut(&mut self.0).expect("Multiple references exist")
    }
}

impl<T> PrivateRc<T> {
    #[must_use]
    pub fn clone_public<'a>(&self) -> PublicRc<'a, T> {
        PublicRc(self.0.clone(), std::marker::PhantomData)
    }
}

pub struct PublicRc<'a, T>(Rc<T>, std::marker::PhantomData<&'a ()>);

impl<'a, T> Deref for PublicRc<'a, T>
where
    T: 'a,
    Self: 'a,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Realm {
    pub intrinsics: PrivateRc<Intrinsics>, // [[Intrinsics]]
    pub global: ObjectHandle,              // [[GlobalObject]]
    pub env: Environment,                  // [[GlobalEnv]]
    pub queue: AsyncTaskQueue,
}

impl Debug for Realm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Realm").finish()
    }
}

impl Realm {
    pub fn new() -> Res<Self> {
        let intrinsics = Intrinsics::new();
        let proto = intrinsics.obj.clone();

        let mut realm = Self {
            intrinsics: PrivateRc(Rc::new(intrinsics)),
            global: new_global_obj(proto)?,
            env: Environment {
                modules: HashMap::new(),
            },
            queue: AsyncTaskQueue::new(),
        };

        init_global_obj(&mut realm)?;

        Ok(realm)
    }

    pub fn set_eval(&mut self, eval: impl Eval + 'static, strict: bool) -> Res {
        let eval_func = NativeFunction::with_len(
            "eval",
            move |args, _, realm| {
                let Some(code) = args.first() else {
                    return Ok(Value::Undefined);
                };

                let code = code.to_string(realm)?;

                let mut scope = Scope::global(realm, PathBuf::from("eval")); //TODO: the scope should be the caller's scope

                //TODO: this is a hack
                if strict {
                    scope = scope.child()?;
                    scope.set_strict_mode()?;
                    scope.state_set_function()?;
                }

                eval.eval(&code, realm, &mut scope)
            },
            self,
            1,
        );

        self.intrinsics.eval = Some(eval_func.clone());
        let global = self.global.clone();
        global.define_property_attributes(
            "eval".into(),
            Variable::write_config(eval_func.into()),
            self,
        )?;

        Ok(())
    }

    pub async fn run_event_loop(&mut self) {
        self.queue.runner().run(self).await;
    }
}

impl Default for Realm {
    fn default() -> Self {
        Self {
            intrinsics: PrivateRc(Rc::new(Intrinsics::default())),
            global: Object::null(),
            env: Environment {
                modules: HashMap::new(),
            },
            queue: AsyncTaskQueue::new(),
        }
    }
}

pub trait Eval {
    fn eval(&self, code: &str, realm: &mut Realm, scope: &mut Scope) -> ValueResult;
}

// impl Eq for Realm {}
