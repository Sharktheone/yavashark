use swc_ecma_ast::ContinueStmt;
use yavashark_env::scope::Scope;
use yavashark_env::{ControlFlow, Realm, RuntimeResult};

use crate::Interpreter;

impl Interpreter {
    pub fn run_continue(
        realm: &mut Realm,
        stmt: &ContinueStmt,
        scope: &mut Scope,
    ) -> RuntimeResult {
        if !scope.state_is_continuable()? {
            return Err(ControlFlow::error_syntax("Illegal continue statement"));
        }

        if let Some(label) = &stmt.label {
            if !scope.has_label(label.sym.as_ref())? {
                return Err(ControlFlow::error_reference(format!(
                    "Label {} not found",
                    label.sym
                )));
            }
        }
        Err(ControlFlow::Continue(
            stmt.label.as_ref().map(|l| l.sym.to_string()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use yavashark_env::{test_eval, Value};

    #[test]
    fn run_continue() {
        test_eval!(
            r"
            let i = 0;
            while (i < 3) {
                i++;
                if (i == 2) {
                    continue;
                }
                mock.values(i);
            }
            ",
            0,
            vec![vec![Value::Number(1.0)], vec![Value::Number(3.0)]],
            Value::Undefined
        );
    }

    #[test]
    fn run_continue_label() {
        test_eval!(
            r"
            let i = 0;
            loop1: while (i < 3) {
                i++;
                let j = 0;
                loop2: while (j < 3) {
                    j++;
                    if (i == 2) {
                        continue loop1;
                    }
                    mock.values(j);
                }
            }
            ",
            0,
            vec![
                vec![Value::Number(1.0)],
                vec![Value::Number(2.0)],
                vec![Value::Number(3.0)],
                vec![Value::Number(1.0)],
                vec![Value::Number(2.0)],
                vec![Value::Number(3.0)]
            ],
            Value::Undefined
        );
    }
    #[test]
    fn run_continue_label2() {
        test_eval!(
            r"
            let i = 0;
            loop1: while (i < 3) {
                i++;
                loop2: while (i < 3) {
                    i++;
                    if (i == 2) {
                        continue loop1;
                    }
                    mock.values(i);
                }
            }
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Undefined
        );
    }
}
