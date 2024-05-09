use crate::context::Context;
use crate::scope::Scope;
use crate::{ControlFlow, Error};
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::DoWhileStmt;

impl Context {
    pub fn run_do_while(&mut self, stmt: &DoWhileStmt, scope: &mut Scope) -> RuntimeResult {
        let mut result = Value::Undefined;

        let last_loop = scope.last_label();
        
        loop {
            
            let scope = &mut Scope::with_parent(scope);
            scope.state_set_loop();
            
            result = match self.run_statement(&stmt.body, scope) {
                Ok(v) => v,
                Err(c) => match c {
                    crate::ControlFlow::Break(l) => {
                        if last_loop.as_ref() == l.as_ref() {
                            break;
                        } else {
                            return Err(ControlFlow::Break(l));
                        }
                    }
                    crate::ControlFlow::Continue(l) => {
                        if last_loop.as_ref() == l.as_ref() {
                            continue;
                        } else {
                            return Err(ControlFlow::Continue(l));
                        }
                    }
                    _ => return Err(c),
                },
                Err(e) => return Err(e),
            
            };
            
            let condition = self.run_expr(&stmt.test, stmt.span, scope)?;

            if condition.is_falsey() {
                break;
            }
        }

        Ok(result)
    }
}



#[cfg(test)]
mod tests {
    use crate::{test_eval, Value};

    #[test]
    fn run_do_while() {
        test_eval!(
            r#"
            let i = 0;
            do {
                i++;
            } while (i < 3);
            i;
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(3.0)
        );
    }
    
    
    #[test]
    fn run_do_while_false() {
        test_eval!(
            r#"
            let i = 0;
            do {
                i++;
            } while (i < 0);
            i;
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(1.0)
        );
    }
    
    #[test]
    fn run_do_while_break() {
        test_eval!(
            r#"
            let i = 0;
            do {
                i++;
                if (i === 2) {
                    break;
                }
            } while (i < 3);
            i;
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(2.0)
        );
    }
    
    #[test]
    fn run_do_while_continue() {
        test_eval!(
            r#"
            let i = 0;
            do {
                i++;
                if (i === 2) {
                    continue;
                }
            } while (i < 3);
            i;
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(3.0)
        );
    }
    
    #[test]
    fn run_do_while_break_and_continue() {
        test_eval!(
            r#"
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
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(3.0)
        );
    }
}