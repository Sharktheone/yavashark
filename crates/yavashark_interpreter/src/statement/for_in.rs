use crate::Interpreter;
use swc_ecma_ast::{ForHead, ForInStmt};
use yavashark_env::scope::Scope;
use yavashark_env::value::Obj;
use yavashark_env::{Context, ControlFlow, Error, RuntimeResult, Value};

impl Interpreter {
    pub fn run_for_in(ctx: &mut Context, stmt: &ForInStmt, scope: &mut Scope) -> RuntimeResult {
        let obj = Self::run_expr(ctx, &stmt.right, stmt.span, scope)?;

        match obj {
            Value::Object(obj) => Self::run_for_in_obj(ctx, &***obj.get()?, stmt, scope),
            _ => Err(Error::ty_error(format!("{obj:?} is not an object")).into()),
        }
    }

    pub fn run_for_in_obj(
        ctx: &mut Context,
        obj: &dyn Obj<Context>,
        stmt: &ForInStmt,
        scope: &mut Scope,
    ) -> RuntimeResult {
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

        for key in obj.keys() {
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
    fn test_for_in() {
        test_eval!(
            "
            let obj = { a: 1, b: 2, c: 3 };
            for (let key in obj) {
                mock.values(key);
            }
            ",
            0,
            vec![
                vec![Value::String("a".into())],
                vec![Value::String("b".into())],
                vec![Value::String("c".into())],
            ],
            Value::Undefined
        ); //TODO: this test not always passes, since it somehow is in random order
    }
}
