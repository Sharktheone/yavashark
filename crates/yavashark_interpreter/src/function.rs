use crate::Interpreter;
use log::info;
use swc_ecma_ast::{BlockStmt, Param, Pat};
use yavashark_env::array::Array;
use yavashark_env::realm::Realm;
use yavashark_env::scope::Scope;
use yavashark_env::{ControlFlow, Error, Object, ObjectHandle, Value, ValueResult, Variable};
use yavashark_macro::object;
use yavashark_value::{Constructor, CustomName, Func, Obj, ObjectProperty};

#[allow(clippy::module_name_repetitions)]
#[object(function, constructor, direct(prototype), name)]
#[derive(Debug)]
pub struct JSFunction {
    pub name: String,
    pub params: Vec<Param>,
    pub block: Option<BlockStmt>,
    #[gc(untyped)]
    pub scope: Scope,
}

impl CustomName for JSFunction {
    fn custom_name(&self) -> String {
        self.name.clone()
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

        let this = Self {
            name,
            params,
            block,
            scope,
            object: Object::raw_with_proto(realm.intrinsics.func.clone().into()),
            prototype: prototype.clone().into(),
        };

        let handle = ObjectHandle::new(this);
        prototype.define_property("constructor".into(), handle.clone().into());

        handle
    }
}

impl Func<Realm> for JSFunction {
    fn call(&mut self, realm: &mut Realm, args: Vec<Value>, this: Value) -> ValueResult {
        let scope = &mut Scope::with_parent(&self.scope)?;
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

impl Constructor<Realm> for JSFunction {
    fn get_constructor(&self) -> ObjectProperty<Realm> {
        self.prototype
            .value
            .get_property_no_get_set(&"constructor".into())
            .unwrap_or(Value::Undefined.into())
    }

    fn value(&self, _realm: &mut Realm) -> Value {
        Object::with_proto(self.prototype.value.clone()).into()
    }

    fn proto(&self, realm: &mut Realm) -> yavashark_value::Value<Realm> {
        self.prototype.value.clone()
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
            r#"
                function foo() {
                    console.log(arguments)
                    for (let arg of arguments) {
                        mock.values(arg)
                    }
                } 
                
                
                foo(1,2,3,4,5)
            "#,
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
