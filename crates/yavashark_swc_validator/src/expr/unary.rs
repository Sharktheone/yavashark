use crate::Validator;
use swc_ecma_ast::{Expr, MemberProp, OptChainBase, UnaryExpr, UnaryOp};

impl<'a> Validator<'a> {
    pub fn validate_unary_expr(&mut self, unary: &'a UnaryExpr) -> Result<(), String> {
        if unary.op == UnaryOp::Delete {
            if Self::delete_target_contains_private(&unary.arg) {
                return Err("Cannot delete private elements".to_string());
            }

            if self.in_strict_mode() && Self::delete_target_is_identifier(&unary.arg) {
                return Err("Cannot delete identifier references in strict mode".to_string());
            }
        }

        self.validate_expr(&unary.arg)
    }

    fn delete_target_contains_private(expr: &Expr) -> bool {
        match expr {
            Expr::Paren(paren) => Self::delete_target_contains_private(&paren.expr),
            Expr::Seq(seq) => seq
                .exprs
                .last()
                .is_some_and(|e| Self::delete_target_contains_private(e)),
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

    fn delete_target_is_identifier(expr: &Expr) -> bool {
        match expr {
            Expr::Ident(_) => true,
            Expr::Paren(paren) => Self::delete_target_is_identifier(&paren.expr),
            Expr::Seq(seq) => seq
                .exprs
                .last()
                .is_some_and(|e| Self::delete_target_is_identifier(e)),
            _ => false,
        }
    }
}
