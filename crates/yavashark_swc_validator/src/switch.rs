use swc_ecma_ast::SwitchStmt;
use crate::Validator;

impl Validator {
    pub fn validate_switch(brk: &SwitchStmt) -> Result<(), String> {
        Self::validate_expr(&brk.discriminant)?;
        for case in &brk.cases {
            if let Some(test) = &case.test {
                Self::validate_expr(test)?;
            }
            Self::validate_statements(&case.cons)?;
        }
        Ok(())
    }
}
