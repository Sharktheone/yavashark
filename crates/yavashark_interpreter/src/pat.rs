use swc_ecma_ast::Pat;

use yavashark_value::Obj;

use crate::{Error, Res, Value};
use crate::context::Context;
use crate::object::array::Array;
use crate::scope::Scope;

impl Context {
    pub fn run_pat(&mut self, stmt: &Pat, scope: &mut Scope, value: Value) -> Res {
        match stmt {
            Pat::Ident(id) => {
                scope.declare_var(id.sym.to_string(), value);
            }
            Pat::Array(arr) => {
                let mut iter = value.iter_no_ctx(self)?;
                
                let mut assert_last = false;
                
                for elem in &arr.elems {
                    if assert_last {
                        return Err(Error::syn("Rest element must be last element"));
                    }
                    
                    let next = iter.next(self)?.unwrap_or(Value::Undefined);

                    if matches!(elem, Some(Pat::Rest(_))) {
                        let rest = elem.as_ref().unwrap(); // Safe to unwrap because of the match above

                        let mut elems = Vec::new();

                        while let Some(res) = iter.next(self)? {
                            elems.push(res);
                        }

                        let elems = Array::from(elems).into_value();

                        self.run_pat(rest, scope, elems)?;
                        let assert_last = true;
                    }

                    if let Some(elem) = elem {
                        self.run_pat(elem, scope, next)?;
                    }
                }
            }
            Pat::Rest(rest) => {
                self.run_pat(&rest.arg, scope, value)?;
            }
            _ => todo!(),
        }

        Ok(())
    }
}
