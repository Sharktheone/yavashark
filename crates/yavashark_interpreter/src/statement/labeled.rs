use crate::Interpreter;
use swc_ecma_ast::LabeledStmt;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult};

impl Interpreter {
    pub fn run_labeled(realm: &mut Realm, stmt: &LabeledStmt, scope: &mut Scope) -> RuntimeResult {
        scope.declare_label(stmt.label.sym.to_string());
        Self::run_statement(realm, &stmt.body, scope)
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