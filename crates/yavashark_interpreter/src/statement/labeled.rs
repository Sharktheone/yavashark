use crate::Interpreter;
use swc_ecma_ast::LabeledStmt;
use yavashark_env::scope::Scope;
use yavashark_env::{ControlFlow, Realm, RuntimeResult, Value};

impl Interpreter {
    pub fn run_labeled(realm: &mut Realm, stmt: &LabeledStmt, scope: &mut Scope) -> RuntimeResult {
        let label = stmt.label.sym.as_str();
        
        scope.declare_label(label.to_string());
        scope.state_set_breakable()?;
        match Self::run_statement(realm, &stmt.body, scope) {
            Err(ControlFlow::Break(l)) if l.as_deref() == Some(label) => Ok(Value::Undefined),
            res => res
        }
    }
}

#[cfg(test)]
mod tests {
    use yavashark_env::{test_eval, Value};

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
}
