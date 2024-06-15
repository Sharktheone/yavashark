use swc_ecma_ast::ReturnStmt;

use yavashark_env::{Context, ControlFlow, RuntimeResult, Value};
use yavashark_env::scope::Scope;

use crate::Interpreter;

impl Interpreter {
    pub fn run_return(ctx: &mut Context, stmt: &ReturnStmt, scope: &mut Scope) -> RuntimeResult {
        if !scope.state_is_returnable()? {
            return Err(ControlFlow::error_syntax("Illegal return statement"));
        }

        let value = if let Some(arg) = &stmt.arg {
            Self::run_expr(ctx, arg, stmt.span, scope)?
        } else {
            Value::Undefined
        };

        Err(ControlFlow::Return(value))
    }
}

#[cfg(test)]
mod tests {
    use yavashark_env::{test_eval, Value};

    #[test]
    fn run_return() {
        test_eval!(
            r"
            function foo(){
                return 1;
            }
            foo();
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(1.0)
        );
    }

    #[test]
    fn run_return_undefined() {
        test_eval!(
            r"
            function foo(){
                return;
            }
            foo();
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Undefined
        );
    }

    #[test]
    fn run_return_no_return() {
        test_eval!(
            r"
            function foo(){
                1;
            }
            foo();
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Undefined
        );
    }

    #[test]
    fn run_return_no_return_null() {
        test_eval!(
            r"
            function foo(){
                null;
            }
            foo();
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Undefined
        );
    }

    #[test]
    fn run_return_no_return_undefined() {
        test_eval!(
            r"
            function foo(){
                mock.send();
                return 1;
                mock.send();
            }
            foo();
            ",
            1,
            Vec::<Vec<Value>>::new(),
            Value::Number(1.0)
        );
    }
}
