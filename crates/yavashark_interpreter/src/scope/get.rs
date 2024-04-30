use std::cell::RefCell;
use std::rc::Rc;

use swc_common::Span;
use swc_ecma_ast::{Ident, MemberExpr, MemberProp, SimpleAssignTarget};

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
                let value = self.get_ident_mut(ident);

                if let Some(value) = value {
                    Ok(AssignValue::MutValue(value))
                } else {
                    Err(Error::reference(format!("{} is not defined", ident.sym.as_str())))
                }
            }

            SimpleAssignTarget::Member(member) => {
                let value = self.get_member_mut(ctx, member.span, member)?;

                Ok(AssignValue::MutObject(value))
            }

            _ => todo!("SimpleAssignTarget")
        }
    }

    pub fn get_ident_mut(&mut self, ident: &Ident) -> Option<MutValue> {
        let ident = ident.sym.to_string();

        self.get_mut(&ident)
    }

    pub fn get_member_mut(&mut self, ctx: &mut Context, span: Span, member: &MemberExpr) -> Result<MutObjectValue, Error> {
        let obj = ctx.run_expr(&member.obj, span, self)?;

        if let Value::Object(obj) = obj {
            let name = match &member.prop {
                MemberProp::Ident(ident) => {
                    ident.sym.to_string()
                }
                MemberProp::Computed(expr) => {
                    let value = ctx.run_expr(&expr.expr, span, self)?;
                    match value {
                        Value::String(s) => s,
                        _ => todo!("Computed property name other than string")
                    }
                }
                _ => todo!("MemberProp")
            };

            Ok(MutObjectValue {
                name,
                obj: Rc::clone(&obj),
            })
        } else {
            Err(Error::new("Cannot assign to a property of a non-object".to_owned()))
        }
    }
}