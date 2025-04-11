use crate::Interpreter;
use std::iter;
use swc_ecma_ast::{ForHead, ForInStmt};
use yavashark_env::scope::Scope;
use yavashark_env::value::Obj;
use yavashark_env::{ControlFlow, Error, Realm, Res, RuntimeResult, Value};

impl Interpreter {
    pub fn run_for_in(realm: &mut Realm, stmt: &ForInStmt, scope: &mut Scope) -> RuntimeResult {
        let obj = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;

        match obj {
            Value::Object(obj) => Self::run_for_in_obj(realm, &**obj.get(), stmt, scope),
            _ => Err(Error::ty_error(format!("{obj:?} is not an object")).into()),
        }
    }

    pub fn run_for_in_obj(
        realm: &mut Realm,
        obj: &dyn Obj<Realm>,
        stmt: &ForInStmt,
        scope: &mut Scope,
    ) -> RuntimeResult {
        for key in obj.keys()? {
            let scope = &mut Scope::with_parent(scope)?;
            let label = scope.last_label()?;
            scope.state_set_loop()?;

            let Some(prop) = obj.get_property(&key)? else {
                //TODO: we should directly return the attributes and so on in `.keys`
                continue;
            };

            if !prop.attributes.is_enumerable() {
                continue;
            }

            Self::run_for_head(realm, stmt.left.clone(), scope, &key)?;

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

    pub fn run_for_head(realm: &mut Realm, decl: ForHead, scope: &mut Scope, value: &Value) -> Res {
        match decl {
            ForHead::VarDecl(decl) => {
                //TODO: respect var decl kind
                for decl in decl.decls {
                    let value = if value.is_truthy() {
                        value.clone()
                    } else if let Some(init) = &decl.init {
                        Self::run_expr(realm, init, decl.span, scope)?
                    } else {
                        value.clone()
                    };

                    Self::run_pat(realm, &decl.name, scope, &mut iter::once(value.clone()))?;
                }
            }
            ForHead::UsingDecl(decl) => {
                for decl in decl.decls {
                    let value = if value.is_truthy() {
                        value.clone()
                    } else if let Some(init) = &decl.init {
                        Self::run_expr(realm, init, decl.span, scope)?
                    } else {
                        value.clone()
                    };

                    Self::run_pat(realm, &decl.name, scope, &mut iter::once(value.clone()))?;
                }
            }
            ForHead::Pat(pat) => {
                Self::run_pat(realm, &pat, scope, &mut iter::once(value.clone()))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yavashark_env::test_eval;

    #[test]
    fn test_for_in() {
        let (result, values) = test_eval!(
            "
            let obj = { a: 1, b: 2, c: 3 };
            for (let key in obj) {
                mock.values(key);
            }
            "
        ); //TODO: this test not always passes, since it somehow is in random order

        assert_eq!(result, Ok(Value::Undefined));

        let values = &values.borrow().got_values;

        assert!(values.contains(&vec![Value::String("a".to_string())]));
        assert!(values.contains(&vec![Value::String("b".to_string())]));
        assert!(values.contains(&vec![Value::String("c".to_string())]));
    }
}
