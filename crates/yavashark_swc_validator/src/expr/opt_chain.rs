use crate::Validator;
use swc_ecma_ast::{OptChainBase, OptChainExpr};

impl<'a> Validator<'a> {
    pub fn validate_opt_chain_expr(&mut self, opt_chain: &'a OptChainExpr) -> Result<(), String> {
        match &*opt_chain.base {
            OptChainBase::Member(member_expr) => {
                self.validate_member_expr(member_expr)?;
            }
            OptChainBase::Call(call) => {
                self.validate_expr(&call.callee)?;

                for arg in &call.args {
                    self.validate_expr(&arg.expr)?;
                }
            }
        }

        Ok(())
    }
}
