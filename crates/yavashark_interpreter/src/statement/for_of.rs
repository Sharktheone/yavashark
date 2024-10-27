use crate::Interpreter;
use swc_ecma_ast::{ForHead, ForOfStmt};
use yavashark_env::scope::Scope;
use yavashark_env::{Context, ControlFlow, RuntimeResult, Value};

impl Interpreter {
    pub fn run_for_of(realm: &mut Realm, stmt: &ForOfStmt, scope: &mut Scope) -> RuntimeResult {
        let obj = Self::run_expr(ctx, &stmt.right, stmt.span, scope)?;

        let scope = &mut Scope::with_parent(scope)?;
        let label = scope.last_label()?;
        scope.state_set_loop()?;

        let ForHead::VarDecl(v) = &stmt.left else {
            todo!("ForInStmt left is not VarDecl");
        };

        if v.decls.is_empty() {
            ControlFlow::error_syntax("ForInStmt left is empty");
        }

        if v.decls.len() > 1 {
            ControlFlow::error_syntax(
                "Invalid left-hand side in for-in loop: Must have a single binding.",
            );
        }

        let decl = v.decls[0]
            .name
            .clone()
            .ident()
            .ok_or_else(|| ControlFlow::error_syntax("ForInStmt left is not an identifier"))?
            .sym
            .to_string();

        let iter = obj.iter_no_ctx(ctx)?;

        while let Some(key) = iter.next(ctx)? {
            scope.declare_var(decl.clone(), key);

            let result = Self::run_statement(ctx, &stmt.body, scope);
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
