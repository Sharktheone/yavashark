use swc_ecma_ast::SwitchStmt;
use yavashark_env::scope::Scope;
use yavashark_env::{ControlFlow, Realm, RuntimeResult, Value};

use crate::Interpreter;

impl Interpreter {
    pub fn run_switch(realm: &mut Realm, stmt: &SwitchStmt, scope: &mut Scope) -> RuntimeResult {
        let discriminant = Self::run_expr(realm, &stmt.discriminant, stmt.span, scope)?;
        let scope = &mut Scope::with_parent(scope)?;
        scope.state_set_breakable()?;

        let mut ret = None;

        let mut default = None;

        for (i, case) in stmt.cases.iter().enumerate() {
            if ret.is_none() {
                if let Some(test) = &case.test {
                    let test = Self::run_expr(realm, test, case.span, scope)?;
                    if discriminant == test {
                    } else {
                        continue;
                    }
                } else {
                    default = Some(i);
                    continue;
                }
            }


            match Self::run_statements(realm, &case.cons, scope) {
                Err(ControlFlow::Break(_)) => return Ok(ret.unwrap_or(Value::Undefined)),
                Err(e) => return Err(e),
                Ok(v) => {
                    if v.is_nullish() {
                        ret = ret.or(Some(v));
                    } else {
                        ret = Some(v);
                    }
                },
            }
        }

        if ret.is_none() {
            if let Some(default_index) = default {
                for case in stmt.cases.iter().skip(default_index) {
                    match Self::run_statements(realm, &case.cons, scope) {
                        Err(ControlFlow::Break(_)) => return Ok(ret.unwrap_or(Value::Undefined)),
                        Err(e) => return Err(e),
                        Ok(v) => {
                            if v.is_nullish() {
                                ret = ret.or(Some(v));
                            } else {
                                ret = Some(v);
                            }
                        },
                    }
                }
            }
        }



        Ok(ret.unwrap_or(Value::Undefined))
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
