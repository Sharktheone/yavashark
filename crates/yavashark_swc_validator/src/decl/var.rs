use crate::{Validator, pat::collect_bound_names};
use swc_ecma_ast::{VarDecl, VarDeclKind};

impl<'a> Validator<'a> {
    pub fn validate_var_decl(&mut self, d: &'a VarDecl) -> Result<(), String> {
        for decl in &d.decls {
            if d.kind != VarDeclKind::Var {
                let mut names = Vec::new();
                collect_bound_names(&decl.name, &mut names);

                for name in names {
                    self.ensure_not_function_param(name)?;
                }
            }

            self.validate_pat_dup(&decl.name, d.kind != VarDeclKind::Var)?;

            if let Some(init) = &decl.init {
                self.validate_expr(init)?;
            }
        }

        Ok(())
    }
}
