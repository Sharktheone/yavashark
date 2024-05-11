use swc_ecma_ast::WhileStmt;

use crate::{ControlFlow, RuntimeResult};
use crate::context::Context;
use crate::scope::Scope;
use crate::Value;

impl Context {
    pub fn run_while(&mut self, stmt: &WhileStmt, scope: &mut Scope) -> RuntimeResult {
        let label = scope.last_label();

        loop {
            let scope = &mut Scope::with_parent(scope);
            scope.state_set_loop();

            let cond = self.run_expr(&stmt.test, stmt.span, scope)?;

            if !cond.is_truthy() {
                break Ok(Value::Undefined);
            }

            let result = self.run_statement(&stmt.body, scope);

            match result {
                Ok(_) => {}
                Err(e) => match &e {
                    ControlFlow::Break(l) if *l == label => {
                        break Ok(Value::Undefined);
                    }
                    ControlFlow::Continue(l)  if *l == label => {
                        continue;
                    }
                    _ => return Err(e),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{test_eval, Value};

    #[test]
    fn while_ten() {
        test_eval!(
            r"
            let i = 0;
            while (i < 10) {
                i++;
                mock.send()
            }
            i
            ",
            10,
            Vec::<Vec<Value>>::new(),
            Value::Number(10.0)
        );
    }

    #[test]
    fn while_break() {
        test_eval!(
            r"
            let i = 0;
            while (i < 10) {
                i++;
                mock.send()
                if (i === 5) {
                    break;
                }
            }
            i
            ",
            5,
            Vec::<Vec<Value>>::new(),
            Value::Number(5.0)
        );
    }

    #[test]
    fn while_continue() {
        test_eval!(
            r"
            let i = 0;
            while (i < 10) {
                i++;
                if (i % 2 === 0) {
                    continue;
                }
                mock.send()
            }
            i
            ",
            5,
            Vec::<Vec<Value>>::new(),
            Value::Number(10.0)
        );
    }

    #[test]
    fn while_nested() {
        test_eval!(
            r"
            let i = 0;
            while (i < 10) {
                i++;
                let j = 0;
                while (j < 10) {
                    j++;
                    mock.send()
                }
            }
            i
            ",
            100,
            Vec::<Vec<Value>>::new(),
            Value::Number(10.0)
        );
    }

    #[test]
    fn while_nested_break() {
        test_eval!(
            r"
            let i = 0;
            while (i < 10) {
                i++;
                let j = 0;
                while (j < 10) {
                    j++;
                    mock.send()
                    if (j === 5) {
                        break;
                    }
                }
            }
            i
            ",
            50,
            Vec::<Vec<Value>>::new(),
            Value::Number(10.0)
        );
    }

    #[test]
    fn while_nested_continue() {
        test_eval!(
            r"
            let i = 0;
            while (i < 10) {
                i++;
                let j = 0;
                while (j < 10) {
                    j++;
                    if (j % 2 === 0) {
                        continue;
                    }
                    mock.send()
                }
            }
            i
            ",
            50,
            Vec::<Vec<Value>>::new(),
            Value::Number(10.0)
        );
    }

    #[test]
    fn while_nested_break_outer() {
        test_eval!(
            r"
            let i = 0;
            while (i < 10) {
                i++;
                let j = 0;
                while (j < 10) {
                    j++;
                    mock.send()
                    if (j === 5) {
                        break;
                    }
                }
                if (i === 5) {
                    break;
                }
            }
            i
            ",
            25,
            Vec::<Vec<Value>>::new(),
            Value::Number(5.0)
        );
    }

    #[test]
    fn while_nested_continue_outer() {
        test_eval!(
            r"
            let i = 0;
            while (i < 10) {
                i++;
                let j = 0;
                while (j < 10) {
                    j++;
                    if (j % 2 === 0) {
                        continue;
                    }
                    mock.send()
                }
                if (i === 5) {
                    continue;
                }
            }
            i
            ",
            50,
            Vec::<Vec<Value>>::new(),
            Value::Number(10.0)
        );
    }

    #[test]
    fn while_nested_break_outer_label() {
        test_eval!(
            r"
            let i = 0;
            outer: while (i < 10) {
                i++;
                let j = 0;
                inner: while (j < 10) {
                    j++;
                    mock.send()
                    if (j === 5) {
                        break outer;
                    }
                }
            }
            i
            ",
            5,
            Vec::<Vec<Value>>::new(),
            Value::Number(1.0)
        );
    }

    #[test]
    fn while_nested_continue_outer_label() {
        test_eval!(
            r"
            let i = 0;
            outer: while (i < 10) {
                i++;
                let j = 0;
                inner: while (j < 10) {
                    j++;
                    if (j % 2 === 0) {
                        continue outer;
                    }
                    mock.send()
                }
            }
            i
            ",
            10,
            Vec::<Vec<Value>>::new(),
            Value::Number(10.0)
        );
    }

    #[test]
    fn while_nested_break_outer_label_inner() {
        test_eval!(
            r"
            let i = 0;
            outer: while (i < 10) {
                i++;
                let j = 0;
                inner: while (j < 10) {
                    j++;
                    mock.send()
                    if (j === 5) {
                        break outer;
                    }
                }
            }
            i
            ",
            5,
            Vec::<Vec<Value>>::new(),
            Value::Number(1.0)
        );
    }

    #[test]
    fn while_nested_continue_outer_label_inner() {
        test_eval!(
            r"
            let i = 0;
            outer: while (i < 10) {
                i++;
                let j = 0;
                inner: while (j < 10) {
                    j++;
                    if (j % 2 === 0) {
                        continue outer;
                    }
                    mock.send()
                }
            }
            i
            ",
            10,
            Vec::<Vec<Value>>::new(),
            Value::Number(10.0)
        );
    }

    #[test]
    fn while_nested_break_inner_label() {
        test_eval!(
            r"
            let i = 0;
            outer: while (i < 10) {
                i++;
                let j = 0;
                inner: while (j < 10) {
                    j++;
                    mock.send()
                    if (j === 5) {
                        break inner;
                    }
                }
            }
            i
            ",
            50,
            Vec::<Vec<Value>>::new(),
            Value::Number(10.0)
        );
    }

    #[test]
    fn while_nested_continue_inner_label() {
        test_eval!(
            r"
            let i = 0;
            outer: while (i < 10) {
                i++;
                let j = 0;
                inner: while (j < 10) {
                    j++;
                    if (j % 2 === 0) {
                        continue inner;
                    }
                    mock.send()
                }
            }
            i
            ",
            50,
            Vec::<Vec<Value>>::new(),
            Value::Number(10.0)
        );
    }

    #[test]
    fn while_nested_break_inner_label_outer() {
        test_eval!(
            r"
            let i = 0;
            outer: while (i < 10) {
                i++;
                let j = 0;
                inner: while (j < 10) {
                    j++;
                    mock.send()
                    if (j === 5) {
                        break outer;
                    }
                }
            }
            i
            ",
            5,
            Vec::<Vec<Value>>::new(),
            Value::Number(1.0)
        );
    }

    #[test]
    fn while_nested_continue_inner_label_outer() {
        test_eval!(
            r"
            let i = 0;
            outer: while (i < 10) {
                i++;
                let j = 0;
                inner: while (j < 10) {
                    j++;
                    if (j % 2 === 0) {
                        continue outer;
                    }
                    mock.send()
                }
            }
            i
            ",
            10,
            Vec::<Vec<Value>>::new(),
            Value::Number(10.0)
        );
    }
}
