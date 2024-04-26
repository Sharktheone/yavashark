use swc_ecma_ast::Lit;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;
use crate::{ControlFlow, RuntimeResult};

impl Context {
    pub fn run_lit(&mut self, stmt: &Lit) -> RuntimeResult {
        Ok(match stmt {
            Lit::Str(s) => Value::String(s.value.as_str().to_owned()),
            Lit::Bool(b) => Value::Boolean(b.value), 
            Lit::Null(_) => Value::Null,
            Lit::Num(n) => Value::Number(n.value),
            Lit::BigInt(_) => todo!(),
            Lit::Regex(_) => todo!(),
            Lit::JSXText(_) => return Err(ControlFlow::error("JSXText is not supported".to_owned()))
        })
    }
}