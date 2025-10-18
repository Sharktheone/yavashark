use crate::params::VMParams;
use crate::{BorrowedVM, VMStateFunctionCode, VM};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use yavashark_bytecode::{BytecodeFunctionCode, BytecodeFunctionParams};
use yavashark_env::builtins::Arguments;
use yavashark_env::scope::Scope;
use yavashark_env::value::{Func, Obj};
use yavashark_env::{ControlFlow, Error, MutObject, ObjectHandle, Realm, Res, Value, ValueResult};
use yavashark_macro::object;
use yavashark_string::YSString;

#[object(function)]
#[derive(Debug)]
pub struct BytecodeFunction {
    code: Rc<BytecodeFunctionCode>,
    scope: Scope,
    params: VMParams,
}

impl BytecodeFunction {
    #[must_use]
    pub fn new(
        code: Rc<BytecodeFunctionCode>,
        scope: Scope,
        realm: &Realm,
        params: BytecodeFunctionParams,
    ) -> Self {
        Self {
            inner: RefCell::new(MutableBytecodeFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone()),
            }),
            code,
            scope,
            params: VMParams::from(params),
        }
    }

    #[must_use]
    pub fn empty(realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutableBytecodeFunction {
                object: MutObject::with_proto(realm.intrinsics.generator_function.clone()),
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

impl Func for BytecodeFunction {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, this: Value) -> ValueResult {
        let scope = &mut Scope::with_parent(&self.scope)?;
        scope.state_set_returnable()?;

        self.params.execute(&args, scope.clone(), realm)?;

        let mut scope = Scope::with_parent(scope)?;
        scope.state_set_function()?;

        let args = Arguments::new(args, Some(this.copy()), realm);

        let args = ObjectHandle::new(args);

        scope.declare_var("arguments".to_string(), args.into(), realm)?;

        let ds = self.code.data_section();

        let mut vm = BorrowedVM::with_scope(&self.code.instructions, &ds, realm, scope);

        match vm.run() {
            Ok(()) => {}
            Err(e) => {
                return match e {
                    ControlFlow::Error(e) => Err(e),
                    ControlFlow::Return(v) => Ok(v),
                    ControlFlow::Break(_) => Err(Error::syn("Illegal break statement")),
                    ControlFlow::Continue(_) => Err(Error::syn("Illegal continue statement")),
                    ControlFlow::Yield(_) => Err(Error::syn("Illegal yield statement")),
                    ControlFlow::Await(_) | ControlFlow::YieldStar(_) => {
                        Err(Error::syn("Illegal await statement"))
                    }
                    ControlFlow::OptChainShortCircuit => Ok(Value::Undefined),
                }
            }
        }

        Ok(vm.acc())
    }
}
