use crate::Validator;
use swc_ecma_ast::{MemberExpr, MemberProp};

impl Validator {
    pub fn validate_member_expr(member: &MemberExpr) -> Result<(), String> {
        Self::validate_expr(&member.obj)?;

        if let MemberProp::Computed(computed) = &member.prop {
            Self::validate_expr(&computed.expr)?;
        }

        Ok(())
    }
}
