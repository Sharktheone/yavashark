use std::fmt::Debug;

use swc_ecma_ast::{BlockStmt, Param, Pat};

use yavashark_value::Func;
use yavashark_value::Obj;

use crate::{ControlFlow, Error, Value, ValueResult};
use crate::context::Context;
use crate::scope::Scope;

type NativeScopedFn = Box<dyn FnMut(Vec<Value>, &mut Scope) -> ValueResult>;
type NativeFn = Box<dyn FnMut(Vec<Value>) -> ValueResult>;

pub enum Function {
    Native(NativeFn),
    NativeScope(NativeScopedFn, Scope),
    JS(Vec<Param>, Option<BlockStmt>, Scope),
}

impl Function {
    pub fn native(f: NativeFn) -> Self {
        Function::Native(f)
    }

    pub fn native_val(f: NativeFn) -> Value {
        let this: Box<dyn Func<Context>> = Box::new(Self::native(f));
        this.into()
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Function]")
    }
}

impl PartialEq for Function {
    fn eq(&self, _other: &Self) -> bool {
        false //TODO
    }
}

impl Obj<Context> for Function {
    fn define_property(&mut self, name: Value, value: Value) {
        todo!()
    }

    fn get_property(&self, name: &Value) -> Option<&Value> {
        todo!()
    }

    fn get_property_mut(&mut self, name: &Value) -> Option<&mut Value> {
        todo!()
    }

    fn name(&self) -> &str {
        todo!()
    }

    fn to_string(&self) -> String {
        todo!()
    }
}

impl Func<Context> for Function {
    fn call(&mut self, ctx: &mut Context, args: Vec<Value>, this: Value) -> ValueResult {
        match self {
            Function::Native(f) => f(args),
            Function::NativeScope(f, scope) => {
                f(args, scope)
            }
            Function::JS(param, block, scope) => {
                for (i, p) in param.iter().enumerate() {
                    let name = match p.pat {
                        Pat::Ident(ref i) => i.sym.to_string(),
                        _ => todo!("call args pat")
                    };


                    scope.declare_var(name, args.get(i).unwrap_or(&Value::Undefined).copy());
                }

                if let Some(block) = block {
                    if let Err(e) = ctx.run_block_this(block, scope, this) {
                        return match e {
                            ControlFlow::Error(e) => Err(e),
                            ControlFlow::Return(v) => Ok(v),
                            ControlFlow::Break(_) => Err(Error::syn("Illegal break statement")),
                            ControlFlow::Continue(_) => Err(Error::syn("Illegal continue statement")),
                        };
                    }
                }
                Ok(Value::Undefined)
            }
        }
    }
}
