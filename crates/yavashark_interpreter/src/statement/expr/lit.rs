use crate::Interpreter;
use swc_ecma_ast::Lit;
use yavashark_env::builtins::RegExp;
use yavashark_env::{ControlFlow, Realm, RuntimeResult, Value};

impl Interpreter {
    pub fn run_lit(realm: &mut Realm, stmt: &Lit) -> RuntimeResult {
        Ok(match stmt {
            Lit::Str(s) => Value::String(s.value.as_str().to_owned()),
            Lit::Bool(b) => Value::Boolean(b.value),
            Lit::Null(_) => Value::Null,
            Lit::Num(n) => Value::Number(n.value),
            Lit::BigInt(b) => Value::BigInt(*b.value.clone()),
            Lit::Regex(r) => Value::Object(RegExp::new_from_str_with_flags(
                realm,
                r.exp.as_str(),
                r.flags.as_str(),
            )?),
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
            Value::String("hello".to_owned())
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
