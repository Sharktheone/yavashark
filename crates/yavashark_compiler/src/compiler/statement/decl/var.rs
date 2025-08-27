use crate::Compiler;
use anyhow::anyhow;
use swc_ecma_ast::{VarDecl, VarDeclKind};
use yavashark_bytecode::data::{DataType, Undefined};
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn decl_var(&mut self, var: &VarDecl) -> crate::Res {
        match var.kind {
            VarDeclKind::Var => {
                for decl in &var.decls {
                    if let Some(init) = &decl.init {
                        let out = self.compile_expr_data_acc(init)?;
                        self.compile_pat_var(&decl.name, out)?
                    } else {
                        self.compile_pat_var(&decl.name, DataType::Undefined(Undefined))?
                    }
                }
            }

            VarDeclKind::Let => {
                for decl in &var.decls {
                    if let Some(init) = &decl.init {
                        let out = self.compile_expr_data_acc(init)?;
                        self.compile_pat_let(&decl.name, out)?
                    } else {
                        self.compile_pat_let(&decl.name, DataType::Undefined(Undefined))?
                    }
                }
            }

            VarDeclKind::Const => {
                for decl in &var.decls {

                    if let Some(init) = &decl.init {
                        let out = self.compile_expr_data_acc(init)?;
                        self.compile_pat_const(&decl.name, out)?
                    } else {
                        return Err(anyhow!("Const declaration without initializer"));
                    }
                }
            }
        }

        Ok(())
    }
}
