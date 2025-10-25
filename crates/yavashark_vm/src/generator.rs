use crate::params::VMParams;
use crate::{GeneratorPoll, ResumableVM, VmState};
use std::cell::RefCell;
use std::fmt::Debug;
use std::path::PathBuf;
use std::rc::Rc;
use yavashark_bytecode::{BytecodeFunctionCode, BytecodeFunctionParams};
use yavashark_env::builtins::Arguments;
use yavashark_env::error::Error;
use yavashark_env::realm::Intrinsic;
use yavashark_env::scope::Scope;
use yavashark_env::value::{Func, IntoValue, Obj};
use yavashark_env::{MutObject, Object, ObjectHandle, Realm, Res, Symbol, Value, ValueResult};
use yavashark_macro::{object, props};
use yavashark_string::YSString;

#[object(function)]
#[derive(Debug)]
pub struct GeneratorFunction {
    code: Rc<BytecodeFunctionCode>,
    scope: Scope,
    params: VMParams,
}

impl GeneratorFunction {
    pub fn new(
        code: Rc<BytecodeFunctionCode>,
        scope: Scope,
        realm: &mut Realm,
        params: BytecodeFunctionParams,
    ) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableGeneratorFunction {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .generator_function
                        .get(realm)?
                        .clone(),
                ),
            }),
            code,
            scope,
            params: VMParams::from(params),
        })
    }

    pub fn empty(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableGeneratorFunction {
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

#[props(intrinsic_name = generator_function)]
impl GeneratorFunction {
    #[prop("length")]
    const LENGTH: usize = 0;

    #[constructor]
    pub fn construct(#[realm] realm: &mut Realm, mut args: Vec<Value>) -> ValueResult {
        let Some(code) = args.pop() else {
            return Ok(Self::empty(realm)?.into_value());
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

impl Func for GeneratorFunction {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        let scope = &mut Scope::with_parent(&self.scope)?;
        scope.state_set_returnable()?;
        scope.set_strict_mode()?;

        self.params.execute(&args, scope.clone(), realm)?;

        let mut scope = Scope::with_parent(scope)?;
        scope.state_set_function()?;

        let args = Arguments::new(args, None, realm)?;

        let args = ObjectHandle::new(args);

        scope.declare_var("arguments".to_string(), args.into(), realm)?;

        let generator = Generator::new(realm, Rc::clone(&self.code), scope)?;

        Ok(generator.into_value())
    }
}

#[object]
pub struct Generator {
    state: RefCell<Option<VmState>>,
}

impl Generator {
    pub fn new(realm: &mut Realm, code: Rc<BytecodeFunctionCode>, scope: Scope) -> Res<Self> {
        let state = VmState::new(code, scope);
        Ok(Self {
            inner: RefCell::new(MutableGenerator {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .generator
                        .get(realm)?
                        .clone(),
                ),
            }),
            state: RefCell::new(Some(state)),
        })
    }

    pub fn init(realm: &mut Realm) -> Res {
        realm.intrinsics.generator_function.set_initializer(GeneratorFunction::initialize);
        realm.intrinsics.generator.set_initializer(Self::initialize);

        Ok(())
    }
}

#[props(intrinsic_name = generator)]
impl Generator {
    pub fn next(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let Some(state) = self.state.take() else {
            let obj = Object::new(realm);

            obj.define_property("done".into(), true.into(), realm)?;
            obj.define_property("value".into(), Value::Undefined, realm)?;

            return Ok(obj);
        };

        let vm = ResumableVM::from_state(state, realm);

        match vm.next() {
            GeneratorPoll::Yield(state, val) => {
                self.state.replace(Some(state));

                let obj = Object::new(realm);

                obj.define_property("done".into(), false.into(), realm)?;
                obj.define_property("value".into(), val, realm)?;

                Ok(obj)
            }
            GeneratorPoll::Ret(res) => {
                let val = res?;

                let obj = Object::new(realm);

                obj.define_property("done".into(), true.into(), realm)?;
                obj.define_property("value".into(), val, realm)?;

                Ok(obj)
            }
        }
    }

    #[prop(Symbol::ITERATOR)]
    #[nonstatic]
    pub const fn iterator(this: Value) -> Value {
        this
    }
}

impl Debug for Generator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Generator").finish()
    }
}
