use std::cell::RefCell;
use std::rc::Rc;

use swc_common::Spanned;
use swc_ecma_ast::{MemberProp, SimpleAssignTarget};

use yavashark_value::error::Error;

use crate::{Object, Value};
use crate::context::Context;
use crate::scope::{MutValue, Scope};

pub struct MutObjectValue {
    pub name: String,
    pub obj: Rc<RefCell<Object>>,
}

pub enum AssignValue {
    MutValue(MutValue),
    MutObject(MutObjectValue),
    OptChainNone,
}

impl Scope {
    pub fn get_assign_target(&mut self, ctx: &mut Context, target: &SimpleAssignTarget) -> Result<AssignValue, Error> {
        match target {
            SimpleAssignTarget::Ident(ident) => {
                let id = ident.sym.to_string();

                let value = self.get_mut(&id);

                if let Some(value) = value {
                    Ok(AssignValue::MutValue(value))
                } else {
                    Err(Error::reference(format!("{id} is not defined")))
                }
            }

            SimpleAssignTarget::Member(member) => {
                let obj = ctx.run_expr(&member.obj, target.span(), self)?;

                if let Value::Object(obj) = obj {
                    let name = match &member.prop {
                        MemberProp::Ident(ident) => {
                            ident.sym.to_string()
                        }
                        MemberProp::Computed(expr) => {
                            let value = ctx.run_expr(&expr.expr, target.span(), self)?;
                            match value {
                                Value::String(s) => s,
                                _ => todo!("Computed property name other than string")
                            }
                        }
                        _ => todo!("MemberProp")
                    };

                    Ok(AssignValue::MutObject(MutObjectValue {
                        name,
                        obj: Rc::clone(&obj),
                    }))
                } else {
                    Err(Error::new("Cannot assign to a property of a non-object".to_owned()))
                }
            }

            _ => todo!("SimpleAssignTarget")
        }
    }
}