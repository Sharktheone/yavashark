use crate::Interpreter;
use swc_ecma_ast::DoWhileStmt;
use yavashark_env::scope::Scope;
use yavashark_env::{ControlFlow, Realm, RuntimeResult, Value};

impl Interpreter {
    pub fn run_do_while(realm: &mut Realm, stmt: &DoWhileStmt, scope: &mut Scope) -> RuntimeResult {
        let mut result = Value::Undefined;

        let last_loop = scope.last_label()?;

        loop {
            let scope = &mut Scope::with_parent(scope)?;
            scope.state_set_loop();

            result = match Self::run_statement(realm, &stmt.body, scope) {
                Ok(v) => v,
                Err(c) => match c {
                    ControlFlow::Break(l) if last_loop.as_ref() == l.as_ref() => {
                        break;
                    }
                    ControlFlow::Continue(l) if last_loop.as_ref() == l.as_ref() => {
                        continue;
                    }
                    _ => return Err(c),
                },
                Err(e) => return Err(e),
            };

            let condition = Self::run_expr(realm, &stmt.test, stmt.span, scope)?;

            if condition.is_falsey() {
                break;
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use yavashark_env::{test_eval, Value};

    #[test]
    fn run_do_while() {
        test_eval!(
            r"
            let i = 0;
            do {
                i++;
            } while (i < 3);
            i;
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(3.0)
        );
    }

    #[test]
    fn run_do_while_false() {
        test_eval!(
            r"
            let i = 0;
            do {
                i++;
            } while (i < 0);
            i;
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(1.0)
        );
    }

    #[test]
    fn run_do_while_break() {
        test_eval!(
            r"
            let i = 0;
            do {
                i++;
                if (i === 2) {
                    break;
                }
            } while (i < 3);
            i;
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(2.0)
        );
    }

    #[test]
    fn run_do_while_continue() {
        test_eval!(
            r"
            let i = 0;
            do {
                i++;
                if (i === 2) {
                    continue;
                }
            } while (i < 3);
            i;
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(3.0)
        );
    }

    #[test]
    fn run_do_while_break_and_continue() {
        test_eval!(
            r"
            let i = 0;
            do {
                i++;
                if (i === 2) {
                    continue;
                }
                if (i === 3) {
                    break;
                }
            } while (i < 5);
            i;
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(3.0)
        );
    }
}
