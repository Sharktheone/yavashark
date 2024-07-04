use crate::Interpreter;
use log::info;
use swc_ecma_ast::{BlockStmt, Param, Pat};
use yavashark_env::scope::Scope;
use yavashark_env::{
    Context, ControlFlow, Error, Object, ObjectHandle, Value, ValueResult, Variable,
};
use yavashark_macro::object;
use yavashark_value::{Constructor, Func, Obj};

#[allow(clippy::module_name_repetitions)]
#[object(function, constructor)]
#[derive(Debug)]
pub struct JSFunction {
    pub name: String,
    pub params: Vec<Param>,
    pub block: Option<BlockStmt>,
    #[gc(untyped)]
    pub scope: Scope,
    #[gc]
    pub prototype: Value,
}

impl JSFunction {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        name: String,
        params: Vec<Param>,
        block: Option<BlockStmt>,
        scope: Scope,
        ctx: &mut Context,
    ) -> ObjectHandle {
        let prototype = Object::new(ctx);

        let this = Self {
            name,
            params,
            block,
            scope,
            object: Object::raw_with_proto(ctx.proto.func.clone().into()),
            prototype: prototype.clone().into(),
        };

        let handle = ObjectHandle::new(this);
        prototype.define_property("constructor".into(), handle.clone().into());

        handle
    }
}

impl Func<Context> for JSFunction {
    fn call(&mut self, ctx: &mut Context, args: Vec<Value>, this: Value) -> ValueResult {
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

        if let Some(block) = &self.block {
            if let Err(e) = Interpreter::run_block_this(ctx, block, scope, this) {
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

impl Constructor<Context> for JSFunction {
    fn get_constructor(&self) -> yavashark_value::Value<Context> {
        self.prototype
            .get_property_no_get_set(&"constructor".into())
            .unwrap_or(Value::Undefined)
    }

    fn value(&self, _ctx: &mut Context) -> Value {
        Object::with_proto(self.prototype.clone()).into()
    }

    fn proto(&self, ctx: &mut Context) -> yavashark_value::Value<Context> {
        self.prototype.clone()
    }
}
