use swc_ecma_ast::VarDecl;
use crate::Validator;

impl Validator {
    pub fn validate_var_decl(decl: &VarDecl) -> Result<(), String> {
        for decl in &decl.decls {
            Self::validate_pat(&decl.name)?;

            if let Some(init) = &decl.init {
                Self::validate_expr(init)?;
            }
        }


        Ok(())

    }
}
