use crate::array::Array;
use crate::scope::Scope;
use crate::{ControlFlow, Error, MutObject, Object, ObjectHandle, ObjectProperty, Realm, Res, RuntimeResult, Value, ValueResult, Variable};
use std::any::Any;
use std::cell::RefCell;
use std::fmt::Debug;
use swc_ecma_ast::{Param, Pat};
use yavashark_garbage::{Collectable, GcRef};
use yavashark_macro::object;
use yavashark_value::{BoxedObj, Constructor, ConstructorFn, CustomGcRefUntyped, CustomName, Func};

#[allow(clippy::module_name_repetitions)]
#[object(function, constructor, direct(prototype), name)]
#[derive(Debug)]
pub struct OptimFunction {
    // #[gc(untyped)] //TODO: this is a memleak!
    pub raw: RawOptimFunction,
}

#[derive(Debug)]
pub struct RawOptimFunction {
    pub name: String,
    pub params: Vec<Param>,
    pub block: Option<RefCell<Box<dyn FunctionCode>>>,
    pub scope: Scope,
}

pub trait FunctionCode: Debug {
    fn call(&self, realm: &mut Realm, scope: &mut Scope, this: Value) -> RuntimeResult;

    fn function_any(&self) -> &dyn Any;
}

impl CustomName for OptimFunction {
    fn custom_name(&self) -> String {
        self.raw.name.clone()
    }
}

impl OptimFunction {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        name: String,
        params: Vec<Param>,
        block: Option<RefCell<Box<dyn FunctionCode>>>,
        scope: Scope,
        realm: &Realm,
    ) -> Res<ObjectHandle> {
        let prototype = Object::new(realm);

        scope.copy_path()?;
        
        let len = params.last().map_or(0, |last| if last.pat.is_rest() {
            params.len() -1
        } else {
            params.len()
        });

        let this = Self {
            inner: RefCell::new(MutableOptimFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone().into()),
                prototype: prototype.clone().into(),
            }),
            raw: RawOptimFunction {
                name,
                params,
                block,
                scope,
            },
        };

        let handle = ObjectHandle::new(this);
        
        handle.define_property("name".into(), handle.clone().into())?;
        handle.define_property("length".into(), len.into())?;
        prototype.define_property("constructor".into(), handle.clone().into())?;

        Ok(handle)
    }

    pub fn new_instance(&self) -> ValueResult {
        let inner = self.inner.try_borrow()?;

        let proto = inner.prototype.value.clone();

        let obj = Object::with_proto(proto);

        Ok(obj.into())
    }
}

impl Func<Realm> for OptimFunction {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, this: Value) -> ValueResult {
        self.raw.call(realm, args, this)
    }
}

impl RawOptimFunction {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, this: Value) -> ValueResult {
        let scope = &mut Scope::with_parent(&self.scope)?;
        scope.state_set_function()?;
        scope.state_set_returnable()?;

        for (i, p) in self.params.iter().enumerate() {
            let Pat::Ident(name) = &p.pat else {
                return Err(Error::syn("Invalid function parameter"));
            };

            scope.declare_var(
                name.sym.to_string(),
                args.get(i).unwrap_or(&Value::Undefined).copy(),
            )?;
        }
        
        let scope = &mut Scope::with_parent(&self.scope)?;
        scope.state_set_function()?;
        scope.state_set_returnable()?;

        let args = Array::with_elements(realm, args)?;

        let args = ObjectHandle::new(args);
        
        args.define_property("callee".into(), this.copy())?;

        args.define_variable("callee".into(), Variable::write_config(this.copy()))?;

        if let Some(block) = &self.block {
            let func = block.borrow();

            return match func.call(realm, scope, this) {
                Err(e) => match e {
                    ControlFlow::Error(e) => Err(e),
                    ControlFlow::Return(v) => Ok(v),
                    ControlFlow::Break(_) => Err(Error::syn("Illegal break statement")),
                    ControlFlow::Continue(_) => Err(Error::syn("Illegal continue statement")),
                    ControlFlow::Yield(_) => Err(Error::syn("Illegal yield statement")),
                    ControlFlow::Await(_) => Err(Error::syn("Illegal await statement")),
                    ControlFlow::OptChainShortCircuit => Ok(Value::Undefined),
                },
                Ok(v) => Ok(v),
            };
        }

        Ok(Value::Undefined)
    }
}

impl CustomGcRefUntyped for RawOptimFunction {
    fn gc_untyped_ref<U: Collectable>(&self) -> Option<GcRef<U>> {
        self.scope.gc_untyped_ref()
    }
}

impl Constructor<Realm> for OptimFunction {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let this = self.new_instance()?;

        self.raw.call(realm, args, this.copy())?;

        Ok(this)
    }

    fn construct_proto(&self) -> Res<ObjectProperty> {
        let inner = self.inner.try_borrow()?;

        Ok(inner.prototype.clone())
    }
}

impl ConstructorFn<Realm> for RawOptimFunction {
    fn gc_untyped_ref(&self) -> Option<GcRef<BoxedObj<Realm>>> {
        self.scope.gc_untyped_ref()
    }

    fn construct(&self, args: Vec<Value>, this: Value, realm: &mut Realm) -> Res {
        self.call(realm, args, this)?;

        Ok(())
    }
}
