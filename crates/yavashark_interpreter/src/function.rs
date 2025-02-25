use crate::Interpreter;
use log::info;
use std::any::Any;
use std::cell::RefCell;
use swc_ecma_ast::{BlockStmt, Param, Pat};
use yavashark_env::array::Array;
use yavashark_env::optimizer::FunctionCode;
use yavashark_env::realm::Realm;
use yavashark_env::scope::Scope;
use yavashark_env::{
    ControlFlow, Error, MutObject, Object, ObjectHandle, Res, Result, RuntimeResult, Value,
    ValueResult, Variable,
};
use yavashark_garbage::{Collectable, GcRef};
use yavashark_macro::object;
use yavashark_value::{
    BoxedObj, Constructor, ConstructorFn, CustomGcRefUntyped, CustomName, Func, Obj, ObjectProperty,
};

#[allow(clippy::module_name_repetitions)]
#[object(function, constructor, direct(prototype), name)]
#[derive(Debug)]
pub struct JSFunction {
    // #[gc(untyped)] //TODO: this is a memleak!
    pub raw: RawJSFunction,
}

#[derive(Debug)]
pub struct RawJSFunction {
    pub name: String,
    pub params: Vec<Param>,
    pub block: Option<BlockStmt>,
    pub scope: Scope,
}

#[derive(Debug)]
pub struct OptimizedJSFunction {
    pub block: BlockStmt,
}

impl FunctionCode for OptimizedJSFunction {
    fn call(&self, realm: &mut Realm, scope: &mut Scope, this: Value) -> RuntimeResult {
        Interpreter::run_block_this(realm, &self.block, scope, this)
    }

    fn function_any(&self) -> &dyn Any {
        self
    }
}

impl CustomName for JSFunction {
    fn custom_name(&self) -> String {
        self.raw.name.clone()
    }
}

impl JSFunction {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        name: String,
        params: Vec<Param>,
        block: Option<BlockStmt>,
        scope: Scope,
        realm: &mut Realm,
    ) -> ObjectHandle {
        let prototype = Object::new(realm);

        scope.copy_path();

        let this = Self {
            inner: RefCell::new(MutableJSFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone().into()),
                prototype: prototype.clone().into(),
            }),
            raw: RawJSFunction {
                name,
                params,
                block,
                scope,
            },
        };

        let handle = ObjectHandle::new(this);
        prototype.define_property("constructor".into(), handle.clone().into());

        handle
    }

    pub fn new_instance(&self, realm: &mut Realm) -> ValueResult {
        let inner = self.inner.try_borrow()?;

        let proto = inner.prototype.value.clone();

        let obj = Object::with_proto(proto);

        Ok(obj.into())
    }
}

impl Func<Realm> for JSFunction {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, this: Value) -> ValueResult {
        self.raw.call(realm, args, this)
    }
}

impl RawJSFunction {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, this: Value) -> ValueResult {
        let scope = &mut Scope::with_parent(&self.scope)?;
        scope.state_set_returnable();

        for (i, p) in self.params.iter().enumerate() {
            let Pat::Ident(name) = &p.pat else {
                return Err(Error::syn("Invalid function parameter"));
            };

            scope.declare_var(
                name.sym.to_string(),
                args.get(i).unwrap_or(&Value::Undefined).copy(),
            );
        }

        let args = Array::with_elements(realm, args)?;

        let args = ObjectHandle::new(args);

        scope.declare_var("arguments".into(), args.into());

        if let Some(block) = &self.block {
            if let Err(e) = Interpreter::run_block_this(realm, block, scope, this) {
                return match e {
                    ControlFlow::Error(e) => Err(e),
                    ControlFlow::Return(v) => Ok(v),
                    ControlFlow::Break(_) => Err(Error::syn("Illegal break statement")),
                    ControlFlow::Continue(_) => Err(Error::syn("Illegal continue statement")),
                    ControlFlow::OptChainShortCircuit => Ok(Value::Undefined),
                };
            }
        }

        Ok(Value::Undefined)
    }
}

impl CustomGcRefUntyped for RawJSFunction {
    fn gc_untyped_ref<U: Collectable>(&self) -> Option<GcRef<U>> {
        self.scope.gc_untyped_ref()
    }
}

impl Constructor<Realm> for JSFunction {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let this = self.new_instance(realm)?;

        self.raw.call(realm, args, this.copy())?;

        Ok(this)
    }

    fn construct_proto(&self) -> Result<ObjectProperty<Realm>> {
        let inner = self.inner.try_borrow()?;

        Ok(inner.prototype.clone())
    }
}

impl ConstructorFn<Realm> for RawJSFunction {
    fn gc_untyped_ref(&self) -> Option<GcRef<BoxedObj<Realm>>> {
        self.scope.gc_untyped_ref()
    }

    fn construct(&self, args: Vec<Value>, this: Value, realm: &mut Realm) -> Res {
        self.call(realm, args, this)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Interpreter;
    use swc_common::DUMMY_SP;
    use swc_ecma_ast::{BlockStmt, Param, Pat};
    use yavashark_env::scope::Scope;
    use yavashark_env::test_eval;

    #[test]
    fn test_function() {
        test_eval!(
            r"
            function add(a, b){
                return a + b;
            }
            add(1, 2)
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(3.0)
        );
    }

    #[test]
    fn test_function_with_scope() {
        test_eval!(
            r"
            let a = 1;
            function add(b){
                return a + b;
            }
            add(2)
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(3.0)
        );
    }

    #[test]
    fn test_function_with_scope_and_block() {
        test_eval!(
            r"
            let a = 1;
            function add(b){
                {
                    let a = 2;
                }
                return a + b;
            }
            add(2)
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(3.0)
        );
    }

    #[test]
    fn attach_arbitrary() {
        test_eval!(
            "
                function foo() {}

                console.log(foo)

                console.log(foo.prototype)

                foo.prototype.a = 1

                console.log(foo.prototype.a)


                foo.prototype.a
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(1.0)
        );
    }

    #[test]
    fn arguments() {
        test_eval!(
            r"
                function foo() {
                    console.log(arguments)
                    for (let arg of arguments) {
                        mock.values(arg)
                    }
                } 
                
                
                foo(1,2,3,4,5)
            ",
            0,
            vec![
                vec![Value::Number(1.0)],
                vec![Value::Number(2.0)],
                vec![Value::Number(3.0)],
                vec![Value::Number(4.0)],
                vec![Value::Number(5.0)]
            ],
            Value::Undefined
        );
    }
}
