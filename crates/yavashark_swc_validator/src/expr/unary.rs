use crate::Validator;
use swc_ecma_ast::{Expr, MemberProp, OptChainBase, UnaryExpr, UnaryOp};

impl<'a> Validator<'a> {
    pub fn validate_unary_expr(&mut self, unary: &'a UnaryExpr) -> Result<(), String> {
        if unary.op == UnaryOp::Delete && Self::delete_target_contains_private(&unary.arg) {
            return Err("Cannot delete private elements".to_string());
        }

        self.validate_expr(&unary.arg)
    }

    fn delete_target_contains_private(expr: &Expr) -> bool {
        match expr {
            Expr::Paren(paren) => Self::delete_target_contains_private(&paren.expr),
            Expr::Seq(seq) => seq
                .exprs
                .last()
                .map_or(false, |e| Self::delete_target_contains_private(e)),
            Expr::Member(member) => matches!(member.prop, MemberProp::PrivateName(_)),
            Expr::OptChain(opt_chain) => {
                if let OptChainBase::Member(member) = &*opt_chain.base {
                    matches!(member.prop, MemberProp::PrivateName(_))
                } else {
                    false
                }
            }
            Expr::PrivateName(_) => true,
            _ => false,
        }
    }
}
