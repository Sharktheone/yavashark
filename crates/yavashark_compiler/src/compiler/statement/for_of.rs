use crate::{Compiler, Res};
use anyhow::anyhow;
use swc_ecma_ast::{ForHead, ForOfStmt, VarDeclKind};
use yavashark_bytecode::ConstValue;
use yavashark_bytecode::data::{Data, DataType, OutputData, OutputDataType};
use yavashark_bytecode::instructions::Instruction;
use yavashark_bytecode::{JmpAddr, jmp::Test};

impl Compiler {
    pub fn compile_for_of(&mut self, f: &ForOfStmt) -> Res {
        let init = self.compile_expr_data_out(&f.right)?;

        let iter = self.data_to_out_or_alloc(init);

        self.instructions.push(Instruction::push_iter(init, iter));

        let loop_start = self.instructions.len();

        self.instructions.push(Instruction::push_loop_scope());

        let inst = match &f.left {
            ForHead::VarDecl(dec) => {
                if dec.decls.len() != 1 {
                    return Err(anyhow!("Invalid left-hand side in for-of loop: Must have a single binding."));
                }

                let decl = &dec.decls[0];

                let var_name = if let Some(ident) = &decl.name.as_ident() {
                    self.alloc_var(&*ident.sym)
                } else {
                    todo!()
                };

                match dec.kind {
                    VarDeclKind::Var => {
                        self.instructions.push(Instruction::decl_empty_var(var_name));

                        let inst = (OutputData::data_type(var_name), self.instructions.len());
                        self.instructions.push(Instruction::jmp_rel(0));

                        inst
                    }
                    VarDeclKind::Let => {
                        self.instructions.push(Instruction::decl_empty_var(var_name));

                        let inst = (OutputData::data_type(var_name), self.instructions.len());
                        self.instructions.push(Instruction::jmp_rel(0));

                        inst
                    }
                    VarDeclKind::Const => {
                        let out = self.alloc_reg_or_stack();

                        let inst = (out, self.instructions.len());

                        self.instructions.push(Instruction::jmp_rel(0));
                        self.instructions.push(Instruction::decl_const(out, var_name));

                        inst
                    }
                }
            },
            _ => todo!()
        };


        self.compile_stmt(&f.body);

        self.instructions.push(Instruction::Jmp(loop_start));


        let loop_end = self.instructions.len();

        self.instructions[inst.1] = Instruction::iter_next_jmp(iter, loop_end, inst.0);

        self.instructions.push(Instruction::pop_scope());

        Ok(())
    }
}
