use swc_ecma_ast::SwitchStmt;
use yavashark_env::scope::Scope;
use yavashark_env::{ControlFlow, Realm, RuntimeResult, Value};

use crate::Interpreter;

impl Interpreter {
    pub fn run_switch(realm: &mut Realm, stmt: &SwitchStmt, scope: &mut Scope) -> RuntimeResult {
        let discriminant = Self::run_expr(realm, &stmt.discriminant, stmt.span, scope)?;
        let scope = &mut Scope::with_parent(scope)?;
        scope.state_set_breakable()?;

        let mut had_pass = false;
        for case in &stmt.cases {
            if !had_pass {
                if let Some(test) = &case.test {
                    let test = Self::run_expr(realm, test, case.span, scope)?;
                    if discriminant == test {
                    } else {
                        continue;
                    }
                }
            }

            had_pass = true;

            if let Err(e) = Self::run_statements(realm, &case.cons, scope) {
                return match &e {
                    ControlFlow::Break(_) => Ok(Value::Undefined),
                    _ => Err(e),
                };
            }
        }

        Ok(Value::Undefined)
    }
}

#[cfg(test)]
mod tests {
    use yavashark_env::{test_eval, Value};

    #[test]
    fn run_switch_case() {
        test_eval!(
            r"
            let a = 1;
            switch(a){
                case 1:
                    a = 2;
                    break;
                case 2:
                    a = 3;
                    break;
                default:
                    a = 4;
            }
            a
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(2.0)
        );
    }

    #[test]
    fn switch_case_with_no_match() {
        test_eval!(
            r"
            let a = 1;
            switch(a){
                case 2:
                    a = 3;
                    break;
                case 3:
                    a = 4;
                    break;
                default:
                    a = 5;
            }
            a
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(5.0)
        );
    }

    #[test]
    fn switch_case_with_multiple_matches() {
        test_eval!(
            r"
            let a = 1;
            switch(a){
                case 1:
                    a = 2;
                case 1:
                    a = 3;
                    break;
                default:
                    a = 4;
            }
            a
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(3.0)
        );
    }

    #[test]
    fn switch_case_with_no_default() {
        test_eval!(
            r"
            let a = 1;
            switch(a){
                case 2:
                    a = 3;
                    break;
                case 3:
                    a = 4;
                    break;
            }
            a
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(1.0)
        );
    }
}
