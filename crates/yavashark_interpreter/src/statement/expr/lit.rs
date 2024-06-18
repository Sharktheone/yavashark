use crate::Interpreter;
use swc_ecma_ast::Lit;
use yavashark_env::{Context, ControlFlow, RuntimeResult, Value};

impl Interpreter {
    pub fn run_lit(ctx: &mut Context, stmt: &Lit) -> RuntimeResult {
        Ok(match stmt {
            Lit::Str(s) => Value::String(s.value.as_str().to_owned()),
            Lit::Bool(b) => Value::Boolean(b.value),
            Lit::Null(_) => Value::Null,
            Lit::Num(n) => Value::Number(n.value),
            Lit::BigInt(_) => todo!(),
            Lit::Regex(_) => todo!(),
            Lit::JSXText(_) => {
                return Err(ControlFlow::error("JSXText is not supported".to_owned()));
            }
        })
    }
}
