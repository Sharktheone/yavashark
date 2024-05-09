use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::Value;
use crate::{ControlFlow, RuntimeResult};
use swc_ecma_ast::ReturnStmt;

impl Context {
    pub fn run_return(&mut self, stmt: &ReturnStmt, scope: &mut Scope) -> RuntimeResult {
        if !scope.state_is_returnable() {
            return Err(ControlFlow::error_syntax("Illegal return statement"));
        }

        let value = if let Some(arg) = &stmt.arg {
            self.run_expr(arg, stmt.span, scope)?
        } else {
            Value::Undefined
        };

        Err(ControlFlow::Return(value))
    }
}



#[cfg(test)]
mod tests {
    use crate::{test_eval, Value};
    
    #[test]
    fn run_return() {
        test_eval!(
            r#"
            function foo(){
                return 1;
            }
            foo();
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(1.0)
        );
    }
    
    #[test]
    fn run_return_undefined() {
        test_eval!(
            r#"
            function foo(){
                return;
            }
            foo();
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::Undefined
        );
    }
    
    #[test]
    fn run_return_no_return() {
        test_eval!(
            r#"
            function foo(){
                1;
            }
            foo();
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::Undefined
        );
    }
    
    #[test]
    fn run_return_no_return_null() {
        test_eval!(
            r#"
            function foo(){
                null;
            }
            foo();
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::Undefined
        );
    }
    
    #[test]
    fn run_return_no_return_undefined() {
        test_eval!(
            r#"
            function foo(){
                mock.send();
                return 1;
                mock.send();
            }
            foo();
            "#,
            1,
            Vec::<Vec<Value>>::new(),
            Value::Number(1.0)
        );
    }
    
}