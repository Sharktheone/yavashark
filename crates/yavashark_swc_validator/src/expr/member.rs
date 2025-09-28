use crate::Validator;
use swc_ecma_ast::{MemberExpr, MemberProp};

impl<'a> Validator<'a> {
    pub fn validate_member_expr(&mut self, member: &'a MemberExpr) -> Result<(), String> {
        self.validate_expr(&member.obj)?;

        if let MemberProp::Computed(computed) = &member.prop {
            self.validate_expr(&computed.expr)?;
        } else if let MemberProp::PrivateName(private_name) = &member.prop {
            self.validate_private_name_expr(private_name)?;
        }

        Ok(())
    }
}
