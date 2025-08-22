use crate::Interpreter;
use swc_ecma_ast::{ForHead, ForOfStmt};
use yavashark_env::scope::Scope;
use yavashark_env::{ControlFlow, Realm, RuntimeResult, Value};

impl Interpreter {
    pub fn run_for_of(realm: &mut Realm, stmt: &ForOfStmt, scope: &mut Scope) -> RuntimeResult {
        let obj = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;

        let scope = &mut Scope::with_parent(scope)?;
        let label = scope.last_label()?;
        scope.state_set_loop()?;

        let iter = obj.iter_no_realm(realm)?;

        while let Some(key) = iter.next(realm)? {
            let scope = &mut Scope::with_parent(scope)?;

            Self::run_for_head(realm, &stmt.left, scope, &key)?;

            let result = Self::run_statement(realm, &stmt.body, scope);
            match result {
                Ok(_) => {}
                Err(ControlFlow::Return(v)) => return Ok(v),
                Err(ControlFlow::Break(l)) if label.as_ref() == l.as_ref() => {
                    break;
                }
                Err(ControlFlow::Continue(l)) if label.as_ref() == l.as_ref() => {
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Ok(Value::Undefined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yavashark_env::test_eval;

    #[test]
    fn run_for_of() {
        test_eval!(
            "
            let arr = [1, 2, 3];
            let sum = 0;
            for (let i of arr) {
                mock.values(i);
                sum += i;
                console.log(sum)
            }
            console.log(sum)
            sum
            ",
            0,
            vec![
                vec![Value::from(1)],
                vec![Value::from(2)],
                vec![Value::from(3)],
            ],
            Value::Number(6.0)
        );
    }
}
