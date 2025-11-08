use crate::Interpreter;
use std::rc::Rc;
use swc_ecma_ast::Lit;
use yavashark_env::builtins::RegExp;
use yavashark_env::value::Obj;
use yavashark_env::{ControlFlow, Error, Realm, RuntimeResult, Value};
use yavashark_string::YSString;

impl Interpreter {
    pub fn run_lit(realm: &mut Realm, stmt: &Lit) -> RuntimeResult {
        Ok(match stmt {
            Lit::Str(s) => Value::String(YSString::from_ref(
                s.value
                    .as_str()
                    .ok_or(Error::new("Invalid wtf-8 surrogate"))?,
            )),
            Lit::Bool(b) => Value::Boolean(b.value),
            Lit::Null(_) => Value::Null,
            Lit::Num(n) => Value::Number(n.value),
            Lit::BigInt(b) => Value::BigInt(Rc::new(*b.value.clone())),
            Lit::Regex(r) => Value::Object(
                RegExp::new_from_str_with_flags(realm, r.exp.as_str(), r.flags.as_str())?
                    .into_object(),
            ),
            Lit::JSXText(_) => {
                return Err(ControlFlow::error("JSXText is not supported".to_owned()));
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use yavashark_env::{test_eval, Value};

    #[test]
    fn run_lit_string() {
        test_eval!(
            r#"
            "hello"
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::String("hello".into())
        );
    }

    #[test]
    fn run_lit_boolean() {
        test_eval!(
            "
            true
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Boolean(true)
        );
    }

    #[test]
    fn run_lit_null() {
        test_eval!(
            "
            null
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Null
        );
    }

    #[test]
    fn run_lit_number() {
        test_eval!(
            "
            1
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(1.0)
        );
    }
}
