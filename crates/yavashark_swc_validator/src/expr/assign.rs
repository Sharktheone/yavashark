use crate::Validator;
use swc_ecma_ast::{AssignExpr, AssignTarget, AssignTargetPat, Pat, SimpleAssignTarget};

impl Validator {
    pub fn validate_assign_expr(assign: &AssignExpr) -> Result<(), String> {
        match &assign.left {
            AssignTarget::Simple(simple) => match simple {
                SimpleAssignTarget::Ident(ident) => Self::validate_ident(&ident.id)?,
                SimpleAssignTarget::Member(member) => Self::validate_member_expr(member)?,
                SimpleAssignTarget::SuperProp(prop) => {
                    Self::validate_super_prop_expr(prop)?;
                }
                SimpleAssignTarget::Paren(paren) => {
                    if paren.expr.is_await_expr() {
                        return Err("Invalid assignment target: await expression".to_string());
                    }


                    Self::validate_expr(&paren.expr)?;
                }
                SimpleAssignTarget::OptChain(optchain) => {
                    Self::validate_opt_chain_expr(optchain)?;
                }
                _ => return Err("Unsupported simple assignment target".to_string()),
            },
            AssignTarget::Pat(pat) => match pat {
                AssignTargetPat::Object(obj) => Self::validate_pat(&Pat::Object(obj.clone()))?,
                AssignTargetPat::Array(arr) => Self::validate_pat(&Pat::Array(arr.clone()))?,
                AssignTargetPat::Invalid(_) => {
                    Err("Invalid assignment target pattern".to_string())?
                }
            },
        }

        Self::validate_expr(&assign.right)
    }
}
