use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};
use crate::builtins::{GcPromise, Promise};
use crate::{Object, ObjectHandle, Realm, Res, Value};
use crate::error_obj::ErrorObj;
use crate::task_queue::{AsyncTask, AsyncTaskQueue};
use crate::realm::resolve::{ModuleFinalizer, ResolveModuleResult};
use crate::scope::Module;
use crate::value::Obj;

pub struct DynamicImport {
    module: Pin<Box<dyn std::future::Future<Output = Res<ModuleFinalizer>>>>,
    promise: GcPromise,
}

impl DynamicImport {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        specifier: &str,
        cur_path: &Path,
        cb: impl FnOnce(String, PathBuf, &mut Realm) -> Res<Module> + 'static,
        realm: &mut Realm
    ) -> Res<ObjectHandle> {

        let promise = Promise::new(realm);

        let module = realm.get_module_async(specifier, cur_path, cb)?;

        Ok(match module {
            ResolveModuleResult::Module(m) => {
                let obj = Value::Object(module_to_object(m)?);

                promise.resolve(&obj, realm)?;

                promise.into_object()
            }
            ResolveModuleResult::Async(fut) => {
                let promise = promise.into_object();


                let prom_downcast = promise.downcast::<Promise>()
                    .ok_or(crate::Error::ty("failed to downcast promise"))?;


                let task = Self {
                    module: fut,
                    promise: prom_downcast,
                };


                AsyncTaskQueue::queue_task(task, realm);

                promise
            }
        })
    }
}

pub enum PromiseOrTask {
    Promise(ObjectHandle),
    Task(DynamicImport),
}


impl AsyncTask for DynamicImport {
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context, realm: &mut Realm) -> Poll<Res> {
        let fut = self.module.as_mut();

        let finalizer = match fut.poll(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Ok(res)) => res,
            Poll::Ready(Err(e)) => {
                let err = ErrorObj::error_to_value(e.clone(), realm);

                if let Err(e) = self.promise.reject(&err, realm) {
                    return Poll::Ready(Err(e));
                }

                return Poll::Ready(Err(e));
            }
        };

        let module = finalizer.finalize(realm)?;

        let obj = module_to_object(&module)?;

        let val = Value::Object(obj);

        Poll::Ready(self.promise.resolve(&val, realm))
    }

    fn run_first_sync(&mut self, _realm: &mut Realm) -> Poll<Res> {
        Poll::Pending
    }
}

pub fn module_to_object(module: &Module) -> Res<ObjectHandle> {
    let obj = Object::null();

    for (key, val) in module.exports.properties()? {
        obj.define_property(key, val)?;
    }

    if let Some(default) = &module.default {
        obj.define_property("default".into(), default.clone().into())?;
    }

    Ok(obj)
}