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
#[cfg(feature = "profiler")]
use std::time::Instant;
pub use initialize::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::rc::Rc;

#[cfg(feature = "profiler")]
use yavashark_profiler::{FileProfileWriter, FrameId, Profile};

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
    #[cfg(feature = "profiler")]
    pub profile: Profile,
    #[cfg(feature = "profiler")]
    profile_writer: Option<Box<FileProfileWriter>>,
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
            #[cfg(feature = "profiler")]
            profile: Profile::new(),
            #[cfg(feature = "profiler")]
            profile_writer: None,
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

                let Value::String(code) = code else {
                    return Ok(code.copy());
                };

                let mut scope = Scope::global(realm, PathBuf::from("eval")); //TODO: the scope should be the caller's scope

                //TODO: this is a hack
                if strict {
                    scope = scope.child()?;
                    scope.set_strict_mode()?;
                    scope.state_set_function()?;
                }

                eval.eval(&code.as_str_lossy(), realm, &mut scope)
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

    #[must_use]
    pub fn has_pending_jobs(&self) -> bool {
        !self.queue.is_empty()
    }

    #[cfg(feature = "profiler")]
    pub fn set_profile_writer(&mut self, writer: FileProfileWriter) {
        self.profile_writer = Some(Box::new(writer));
    }

    #[cfg(feature = "profiler")]
    pub fn profile_writer(&self) -> Option<&FileProfileWriter> {
        self.profile_writer.as_deref()
    }

    #[cfg(feature = "profiler")]
    pub fn profile_add_frame(&mut self, fn_name: String, start: Instant) -> FrameId {
        self.profile.add_frame(fn_name, start)
    }

    #[cfg(feature = "profiler")]
    pub fn profile_end_frame(&mut self, frame_id: FrameId, end: Instant) {
        self.profile.end_frame(frame_id, end);
    }

    #[cfg(feature = "profiler")]
    pub fn write_profile(&mut self) -> std::io::Result<Option<std::path::PathBuf>> {
        let Some(writer) = self.profile_writer.as_mut() else {
            return Ok(None);
        };

        writer.write_to_path(self.profile.take()).map(Some)
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
            #[cfg(feature = "profiler")]
            profile: Profile::new(),
            #[cfg(feature = "profiler")]
            profile_writer: None,
        }
    }
}

pub trait Eval {
    fn eval(&self, code: &str, realm: &mut Realm, scope: &mut Scope) -> ValueResult;
}

// impl Eq for Realm {}
