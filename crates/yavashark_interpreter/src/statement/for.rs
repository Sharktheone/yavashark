use crate::Interpreter;
use swc_ecma_ast::{ForStmt, VarDeclOrExpr};
use yavashark_env::scope::Scope;
use yavashark_env::{ControlFlow, Realm, RuntimeResult, Value};

impl Interpreter {
    pub fn run_for(realm: &mut Realm, stmt: &ForStmt, scope: &mut Scope) -> RuntimeResult {
        let scope = &mut Scope::with_parent(scope)?;
        let label = scope.last_label()?;
        scope.state_set_loop()?;

        if let Some(init) = &stmt.init {
            match init {
                VarDeclOrExpr::VarDecl(v) => {
                    Self::decl_var(realm, v, scope)?;
                }
                VarDeclOrExpr::Expr(e) => {
                    Self::run_expr(realm, e, stmt.span, scope)?;
                }
            }
        }

        loop {
            if let Some(test) = &stmt.test {
                let value = Self::run_expr(realm, test, stmt.span, scope)?;
                if value.is_falsey() {
                    break Ok(Value::Undefined);
                }
            }

            if let Err(e) = Self::run_statement(realm, &stmt.body, scope) {
                match e {
                    ControlFlow::Break(l) if label.as_ref() == l.as_ref() => {
                        break Ok(Value::Undefined);
                    }
                    ControlFlow::Continue(l) if label.as_ref() == l.as_ref() => {
                        if let Some(update) = &stmt.update {
                            Self::run_expr(realm, update, stmt.span, scope)?;
                        }
                        continue;
                    }
                    (c) => return Err(c),
                }
            }

            if let Some(update) = &stmt.update {
                Self::run_expr(realm, update, stmt.span, scope)?;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use yavashark_env::{test_eval, Value};

    #[test]
    fn run_for_loop() {
        test_eval!(
            r"
            let a = 0;
            for (let i = 0; i < 10; i++) {
                a++;
                mock.send();
            }
            a
            ",
            10,
            Vec::<Vec<Value>>::new(),
            Value::Number(10.0)
        );
    }

    #[test]
    fn run_for_loop_with_break() {
        test_eval!(
            r"
            let a = 0;
            for (let i = 0; i < 5; i++) {
                if (i === 2) {
                    break;
                }
                mock.send();
                a++;
            }
            a
            ",
            2,
            Vec::<Vec<Value>>::new(),
            Value::Number(2.0)
        );
    }

    #[test]
    fn run_for_loop_with_continue() {
        test_eval!(
            r"
            let a = 0;
            for (let i = 0; i < 5; i++) {
                if (i === 2) {
                    continue;
                }
                a++;
                mock.send();
            }
            a
            ",
            4,
            Vec::<Vec<Value>>::new(),
            Value::Number(4.0)
        );
    }

    #[test]
    fn run_for_loop_with_break_and_continue() {
        test_eval!(
            r"
            let a = 0;
            for (let i = 0; i < 5; i++) {
                if (i === 3) {
                    break;
                }
                
                a++;
                
                if (i === 2) {
                    continue;
                }
                mock.send();
            }
            a
            ",
            2,
            Vec::<Vec<Value>>::new(),
            Value::Number(3.0)
        );
    }
}
