use swc_ecma_ast::{OptChainBase, OptChainExpr};
use crate::Validator;

impl Validator {
    pub fn validate_opt_chain_expr(opt_chain: &OptChainExpr) -> Result<(), String> {
        match &*opt_chain.base {
            OptChainBase::Member(member_expr) => {
                Self::validate_member_expr(member_expr)?;
            }
            OptChainBase::Call(call) => {
                Self::validate_expr(&call.callee)?;
                
                for arg in &call.args {
                    Self::validate_expr(&arg.expr)?;
                }
            }
        }


        Ok(())
    }
}
