use crate::Validator;
use swc_ecma_ast::SwitchStmt;

impl<'a> Validator<'a> {
    pub fn validate_switch(&mut self, brk: &'a SwitchStmt) -> Result<(), String> {
        self.validate_expr(&brk.discriminant)?;
        for case in &brk.cases {
            if let Some(test) = &case.test {
                self.validate_expr(test)?;
            }
            self.validate_statements(&case.cons)?;
        }
        Ok(())
    }
}
