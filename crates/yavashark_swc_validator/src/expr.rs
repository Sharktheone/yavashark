use swc_ecma_ast::{Expr, ExprStmt};
use crate::Validator;

impl Validator {
    pub fn validate_expr_stmt(expr: &ExprStmt) -> Result<(), String> {
        Self::validate_expr(&expr.expr)
    }
    
    pub fn validate_expr(expr: &Expr) -> Result<(), String> {
        Ok(())
    }
}
