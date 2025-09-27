use crate::Validator;
use swc_ecma_ast::{VarDecl, VarDeclKind};

impl Validator {
    pub fn validate_var_decl(d: &VarDecl) -> Result<(), String> {
        for decl in &d.decls {
            Self::validate_pat_dup(&decl.name, d.kind != VarDeclKind::Var)?;

            if let Some(init) = &decl.init {
                Self::validate_expr(init)?;
            }
        }

        Ok(())
    }
}
