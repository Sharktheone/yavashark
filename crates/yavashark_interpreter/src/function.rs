use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use swc_ecma_ast::{BlockStmt, Param, Pat};
use yavashark_value::error::Error;
use yavashark_value::Func;

use crate::{ControlFlow, Object, Value, ValueResult};
use crate::context::Context;
use crate::scope::{Scope, ScopeInternal};

type NativeFn = Box<dyn FnMut(Vec<Value>, &mut Scope) -> ValueResult>;

pub enum Function {
    Native(NativeFn),
    NativeScope(NativeFn, Rc<RefCell<ScopeInternal>>),
    JS(Vec<Param>, Option<BlockStmt>),
}

impl Function {
    pub fn call(&mut self, ctx: &mut Context, args: Vec<Value>, scope: &mut Scope) -> ValueResult {
        match self {
            Function::Native(f) => f(args, scope),
            Function::NativeScope(f, s) => {
                let mut scope = Scope::from(Rc::clone(s));
                f(args, &mut scope)
            }
            Function::JS(param, block) => {
                let mut scope = Scope::with_parent(scope);

                for (i, p) in param.iter().enumerate() {
                    
                    let name = match p.pat { 
                        Pat::Ident(ref i) => i.sym.to_string(),
                        _ => todo!("call args pat")
                    };
                    
                    
                    scope.declare_var(name, args.get(i).unwrap_or(&Value::Undefined).copy());
                }

                if let Some(block) = block {
                    if let Err(e) = ctx.run_block(&block, &mut scope) {
                        return match e {
                            ControlFlow::Error(e) => Err(e),
                            ControlFlow::Return(v) => Ok(v),
                            ControlFlow::Break(_) => Err(Error::syntax("Illegal break statement")),
                            ControlFlow::Continue(_) => Err(Error::syntax("Illegal continue statement")),
                        }
                    }
                }
                Ok(Value::Undefined)
            }
        }
    }

    pub fn native(f: NativeFn) -> Self {
        Function::Native(f)
    }

    pub fn native_val(f: NativeFn) -> Value {
        let obj = Function::native(f).into();
        let ohj = Rc::new(RefCell::new(obj));
        Value::Object(ohj)
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

impl Func for Function {}
