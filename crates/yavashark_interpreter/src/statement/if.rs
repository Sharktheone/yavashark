use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::IfStmt;

impl Context {
    pub fn run_if(&mut self, stmt: &IfStmt, scope: &mut Scope) -> RuntimeResult {
        let test = self.run_expr(&stmt.test, stmt.span, scope)?;

        if test.is_truthy() {
            self.run_statement(&stmt.cons, scope)
        } else if let Some(alt) = &stmt.alt {
            self.run_statement(alt, scope)
        } else {
            Ok(Value::Undefined)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{test_eval, Value};

    #[test]
    fn run_if_true() {
        test_eval!(
            r#"
            if (true) {
                mock.values(1);
            }
            "#,
            0,
            vec![vec![Value::Number(1.0)]],
            Value::Undefined
        );
    }

    #[test]
    fn run_if_false() {
        test_eval!(
            r#"
            if (false) {
                mock.values(1);
            }
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::Undefined
        );
    }

    #[test]
    fn run_if_else() {
        test_eval!(
            r#"
            if (false) {
                mock.values(1);
            } else {
                mock.values(2);
            };
            "#,
            0,
            vec![vec![Value::Number(2.0)]],
            Value::Undefined
        );
    }

    #[test]
    fn run_if_else_if() {
        test_eval!(
            r#"
            if (false) {
                mock.values(1);
            } else if (true) {
                mock.values(2);
            };
            "#,
            0,
            vec![vec![Value::Number(2.0)]],
            Value::Undefined
        );
    }

    #[test]
    fn run_if_else_if_else() {
        test_eval!(
            r#"
            if (false) {
                mock.values(1);
            } else if (false) {
                mock.values(2);
            } else {
                mock.values(3);
            };
            "#,
            0,
            vec![vec![Value::Number(3.0)]],
            Value::Undefined
        );
    }

    #[test]
    fn run_if_truthy_number() {
        test_eval!(
            r#"
            if (1) {
                mock.values(1);
            }
            "#,
            0,
            vec![vec![Value::Number(1.0)]],
            Value::Undefined
        );
    }

    #[test]
    fn run_if_falsy_number() {
        test_eval!(
            r#"
            if (0) {
                mock.values(1);
            }
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::Undefined
        );
    }

    #[test]
    fn run_if_truthy_string() {
        test_eval!(
            r#"
            if ("") {
                mock.values(1);
            }
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::Undefined
        );
    }

    #[test]
    fn run_if_falsy_string() {
        test_eval!(
            r#"
            if ("a") {
                mock.values(1);
            }
            "#,
            0,
            vec![vec![Value::Number(1.0)]],
            Value::Undefined
        );
    }
}
