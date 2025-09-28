use crate::Validator;
use swc_ecma_ast::{VarDecl, VarDeclKind};

impl<'a> Validator<'a> {
    pub fn validate_var_decl(&mut self, d: &'a VarDecl) -> Result<(), String> {
        for decl in &d.decls {
            self.validate_pat_dup(&decl.name, d.kind != VarDeclKind::Var)?;

            if let Some(init) = &decl.init {
                self.validate_expr(init)?;
            }
        }

        Ok(())
    }
}
