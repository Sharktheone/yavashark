use swc_ecma_ast::Lit;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_lit(&mut self, stmt: &Lit) -> Result<Value, Error> {
        Ok(match stmt {
            Lit::Str(s) => Value::String(s.value.as_str().to_owned()),
            Lit::Bool(b) => Value::Boolean(b.value), 
            Lit::Null(_) => Value::Null,
            Lit::Num(n) => Value::Number(n.value),
            Lit::BigInt(_) => todo!(),
            Lit::Regex(_) => todo!(),
            Lit::JSXText(_) => return Err(Error::new("JSXText is not supported".to_owned()))
        })
    }
}