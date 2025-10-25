use crate::params::VMParams;
use crate::task::BytecodeAsyncTask;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use yavashark_bytecode::{BytecodeFunctionCode, BytecodeFunctionParams};
use yavashark_env::builtins::Arguments;
use yavashark_env::scope::Scope;
use yavashark_env::value::{Func, Obj};
use yavashark_env::{MutObject, ObjectHandle, Realm, Res, Value, ValueResult};
use yavashark_macro::object;
use yavashark_string::YSString;

#[object(function)]
#[derive(Debug)]
pub struct AsyncBytecodeFunction {
    code: Rc<BytecodeFunctionCode>,
    scope: Scope,
    params: VMParams,
}

impl AsyncBytecodeFunction {
    #[must_use]
    pub fn new(
        code: Rc<BytecodeFunctionCode>,
        scope: Scope,
        realm: &Realm,
        params: BytecodeFunctionParams,
    ) -> Self {
        Self {
            inner: RefCell::new(MutableAsyncBytecodeFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone()),
            }),
            code,
            scope,
            params: VMParams::from(params),
        }
    }

    pub fn empty(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableAsyncBytecodeFunction {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .generator_function
                        .get(realm)?
                        .clone(),
                ),
            }),
            code: Rc::new(BytecodeFunctionCode::default()),
            scope: Scope::new(realm, PathBuf::new()),
            params: VMParams::default(),
        })
    }

    pub fn update_name(&self, n: &str, realm: &mut Realm) -> Res {
        let name = self
            .resolve_property("name".into(), realm)
            .ok()
            .flatten()
            .and_then(|v| v.assert_value().value.to_string(realm).ok())
            .unwrap_or_default();

        if name.is_empty() {
            self.inner
                .try_borrow_mut()?
                .object
                .force_update_property_cb("name".into(), |v| {
                    if let Some(v) = v {
                        if !v.value.is_string() {
                            return None;
                        }
                    }

                    Some(YSString::from_ref(n).into())
                })?;
        }

        Ok(())
    }
}

impl Func for AsyncBytecodeFunction {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        let scope = &mut Scope::with_parent(&self.scope)?;
        scope.state_set_returnable()?;

        self.params.execute(&args, scope.clone(), realm)?;

        let mut scope = Scope::with_parent(scope)?;
        scope.state_set_function()?;

        let args = Arguments::new(args, None, realm)?;

        let args = ObjectHandle::new(args);

        scope.declare_var("arguments".to_string(), args.into(), realm)?;

        Ok(BytecodeAsyncTask::new(Rc::clone(&self.code), realm, scope)?.into())
    }
}
