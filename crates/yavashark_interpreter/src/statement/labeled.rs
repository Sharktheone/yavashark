use crate::Interpreter;
use swc_ecma_ast::{LabeledStmt, Stmt};
use yavashark_env::scope::Scope;
use yavashark_env::{ControlFlow, Realm, RuntimeResult, Value};

impl Interpreter {
    pub fn run_labeled(realm: &mut Realm, stmt: &LabeledStmt, scope: &mut Scope) -> RuntimeResult {
        let label = stmt.label.sym.as_str();

        scope.declare_label(label.to_string());

        if !matches!(
            *stmt.body,
            Stmt::For(_)
                | Stmt::ForIn(_)
                | Stmt::ForOf(_)
                | Stmt::While(_)
                | Stmt::DoWhile(_)
                | Stmt::Labeled(_)
        ) {
            scope.set_no_label()?;
        }
        scope.state_set_breakable()?;
        match Self::run_statement(realm, &stmt.body, scope) {
            Err(ControlFlow::Break(l)) if l.as_deref() == Some(label) => Ok(Value::Undefined),
            res => res,
        }
    }
}

#[cfg(test)]
mod tests {
    use yavashark_env::{Value, test_eval};

    #[test]
    fn labeled_continue() {
        test_eval!(
            r"
            var count = 0;
            label: for (let x = 0; x < 10;) {
                while (true) {
                    x++;
                    count++;
                    continue label;
                }
            }
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Undefined
        );
    }

    #[test]
    fn labeled_block_break_through_switch() {
        test_eval!(
            r"
            let out = 0;
            a: {
                switch (1) {
                    case 1:
                        break a;
                }
                out = 1;
            }
            out
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(0.0)
        );
    }

    #[test]
    fn labeled_block_break_through_loop() {
        test_eval!(
            r"
            let out = 0;
            a: {
                for (;;) {
                    break a;
                }
                out = 1;
            }
            out
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(0.0)
        );
    }
}
