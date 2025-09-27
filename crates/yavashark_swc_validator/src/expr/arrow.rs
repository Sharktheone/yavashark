use crate::Validator;
use swc_ecma_ast::{ArrowExpr, BlockStmtOrExpr};

impl Validator {
    pub fn validate_arrow_expr(arrow: &ArrowExpr) -> Result<(), String> {
        for param in &arrow.params {
            Self::validate_pat_dup(param, true)?;
        }

        match &*arrow.body {
            BlockStmtOrExpr::BlockStmt(block) => Self::validate_block(block),
            BlockStmtOrExpr::Expr(expr) => Self::validate_expr(expr),
        }
    }
}
