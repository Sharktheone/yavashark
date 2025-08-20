use crate::Compiler;
use anyhow::anyhow;
use swc_ecma_ast::{VarDecl, VarDeclKind};
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn decl_var(&mut self, var: &VarDecl) -> crate::Res {
        match var.kind {
            VarDeclKind::Var => {
                for decl in &var.decls {
                    let name = decl.name.as_ident().ok_or_else(|| todo!())?;
                    let name = self.alloc_var(name.id.as_ref());

                    if let Some(init) = &decl.init {
                        let out = self.compile_expr_data_acc(init)?;
                        self.instructions.push(Instruction::decl_var(out, name));
                    } else {
                        self.instructions.push(Instruction::decl_empty_var(name));
                    }
                }
            }

            VarDeclKind::Let => {
                for decl in &var.decls {
                    let name = decl.name.as_ident().ok_or_else(|| todo!())?;
                    let name = self.alloc_var(name.id.as_ref());

                    if let Some(init) = &decl.init {
                        let out = self.compile_expr_data_acc(init)?;
                        self.instructions.push(Instruction::decl_let(out, name));
                    } else {
                        self.instructions.push(Instruction::decl_empty_let(name));
                    }
                }
            }

            VarDeclKind::Const => {
                for decl in &var.decls {
                    let name = decl.name.as_ident().ok_or_else(|| todo!())?;
                    let name = self.alloc_var(name.id.as_ref());

                    if let Some(init) = &decl.init {
                        let out = self.compile_expr_data_acc(init)?;
                        self.instructions.push(Instruction::decl_const(out, name));
                    } else {
                        return Err(anyhow!("Const declaration without initializer"));
                    }
                }
            }
        }

        Ok(())
    }
}
