mod task;

use crate::async_generator::task::AsyncGeneratorTask;
use crate::params::VMParams;
use crate::VmState;
use std::cell::RefCell;
use std::fmt::Debug;
use std::path::PathBuf;
use std::rc::Rc;
use tokio::sync::Notify;
use yavashark_bytecode::{BytecodeFunctionCode, BytecodeFunctionParams};
use yavashark_env::builtins::Arguments;
use yavashark_env::conversion::downcast_obj;
use yavashark_env::error::Error;
use yavashark_env::scope::Scope;
use yavashark_env::value::{Func, IntoValue, Obj};
use yavashark_env::{MutObject, Object, ObjectHandle, Realm, Res, Symbol, Value, ValueResult};
use yavashark_macro::{object, props};
use yavashark_string::YSString;

#[object(function)]
#[derive(Debug)]
pub struct AsyncGeneratorFunction {
    code: Rc<BytecodeFunctionCode>,
    scope: Scope,
    params: VMParams,
}

impl AsyncGeneratorFunction {
    #[must_use]
    pub fn new(
        code: Rc<BytecodeFunctionCode>,
        scope: Scope,
        realm: &Realm,
        params: BytecodeFunctionParams,
    ) -> Self {
        Self {
            inner: RefCell::new(MutableAsyncGeneratorFunction {
                object: MutObject::with_proto(realm.intrinsics.async_generator_function.clone()),
            }),
            code,
            scope,
            params: VMParams::from(params),
        }
    }

    #[must_use]
    pub fn empty(realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutableAsyncGeneratorFunction {
                object: MutObject::with_proto(realm.intrinsics.async_generator_function.clone()),
            }),
            code: Rc::new(BytecodeFunctionCode::default()),
            scope: Scope::new(realm, PathBuf::new()),
            params: VMParams::default(),
        }
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

#[props]
impl AsyncGeneratorFunction {
    #[prop("length")]
    const LENGTH: usize = 0;

    #[constructor]
    pub fn construct(#[realm] realm: &mut Realm, mut args: Vec<Value>) -> ValueResult {
        let Some(code) = args.pop() else {
            return Ok(Self::empty(realm).into_value());
        };

        let mut buf = "function* anonymous(".to_owned();

        for (i, arg) in args.iter().enumerate() {
            if i != 0 {
                buf.push(',');
            }

            buf.push_str(&arg.to_string(realm)?);
        }

        buf.push_str(") { ");

        buf.push_str(&code.to_string(realm)?);

        buf.push_str(" }");

        buf.push_str("anonymous");

        let Some(eval) = realm.intrinsics.eval.clone() else {
            return Err(Error::new("eval is not defined"));
        };

        eval.call(vec![Value::String(buf.into())], Value::Undefined, realm)
    }
}

impl Func for AsyncGeneratorFunction {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        let scope = &mut Scope::with_parent(&self.scope)?;
        scope.state_set_returnable()?;
        scope.set_strict_mode()?;

        self.params.execute(&args, scope.clone(), realm)?;

        // for (i, p) in self.params.iter().enumerate() {
        //     let Pat::Ident(name) = &p.pat else { todo!() };
        //
        //     scope.declare_var(
        //         name.sym.to_string(),
        //         args.get(i).unwrap_or(&Value::Undefined).copy(),
        //     )?;
        // }

        let mut scope = Scope::with_parent(scope)?;
        scope.state_set_function()?;

        let args = Arguments::new(args, None, realm);

        let args = ObjectHandle::new(args);

        scope.declare_var("arguments".to_string(), args.into(), realm)?;

        let generator = AsyncGenerator::new(realm, Rc::clone(&self.code), scope);

        Ok(generator.into_value())
    }
}

#[object]
pub struct AsyncGenerator {
    state: RefCell<Option<VmState>>,
    notify: Notify,
}

impl AsyncGenerator {
    #[must_use]
    pub fn new(realm: &Realm, code: Rc<BytecodeFunctionCode>, scope: Scope) -> Self {
        let state = VmState::new(code, scope);
        Self {
            inner: RefCell::new(MutableAsyncGenerator {
                object: MutObject::with_proto(realm.intrinsics.async_generator.clone()),
            }),
            state: RefCell::new(Some(state)),
            notify: Notify::new(),
        }
    }

    pub fn init(realm: &mut Realm) -> Res {
        let gf = AsyncGeneratorFunction::initialize_proto(
            Object::raw_with_proto(realm.intrinsics.obj.clone()),
            realm.intrinsics.func.clone().into(),
            realm,
        )?;

        let g = Self::initialize_proto(
            Object::raw_with_proto(realm.intrinsics.obj.clone()),
            realm.intrinsics.func.clone().into(),
            realm,
        )?;

        realm.intrinsics.async_generator_function = gf;
        realm.intrinsics.async_generator = g;

        Ok(())
    }
}

#[props]
impl AsyncGenerator {
    #[nonstatic]
    pub fn next(this: Value, realm: &mut Realm) -> ValueResult {
        let this = downcast_obj::<Self>(this)?;

        let mut state_ref = this.state.try_borrow_mut()?;
        let state = state_ref.take();
        drop(state_ref);

        Ok(AsyncGeneratorTask::new(realm, state, this)?.into())
    }

    #[prop(Symbol::ITERATOR)]
    #[nonstatic]
    pub const fn iterator(this: Value) -> Value {
        this
    }

    #[prop(Symbol::ASYNC_ITERATOR)]
    #[nonstatic]
    pub const fn async_iterator(this: Value) -> Value {
        this
    }
}

impl Debug for AsyncGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Generator").finish()
    }
}
