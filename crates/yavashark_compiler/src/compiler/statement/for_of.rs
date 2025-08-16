use crate::{Compiler, Res};
use anyhow::anyhow;
use swc_ecma_ast::{ForHead, ForOfStmt, VarDeclKind};
use yavashark_bytecode::ConstValue;
use yavashark_bytecode::data::{DataType, OutputDataType};
use yavashark_bytecode::instructions::Instruction;
use yavashark_bytecode::{JmpAddr, jmp::Test};

impl Compiler {
    pub fn compile_for_of(&mut self, f: &ForOfStmt) -> Res {
        // // Evaluate the right-hand side (iterable)
        // let iterable = self.compile_expr_data_acc(&f.right)?;
        //
        // // Get iterator via obj[Symbol.iterator]()
        // let sym_iter = self.alloc_const(ConstValue::Symbol(Symbol));
        // let iter_obj_out = self.alloc_reg_or_stack();
        // // No args to push; directly call the member
        // self.instructions
        //     .push(Instruction::call_member(iterable, sym_iter, iter_obj_out));
        //
        // // If the left is a declaration, declare the variable(s) before the loop (simple ident only)
        // // We keep the VarName to assign into each iteration
        // let mut target_out: Option<OutputDataType> = None;
        // match &f.left {
        //     ForHead::VarDecl(vd) => {
        //         // Only support a single simple identifier for now
        //         if vd.decls.len() != 1 {
        //             return Err(anyhow!(
        //                 "Only a single declaration is supported in for...of"
        //             ));
        //         }
        //         let decl = &vd.decls[0];
        //         let ident = decl
        //             .name
        //             .as_ident()
        //             .ok_or_else(|| anyhow!("Only simple identifiers are supported in for...of"))?;
        //         let var = self.alloc_var(ident.id.as_ref());
        //
        //         match vd.kind {
        //             VarDeclKind::Var => self.instructions.push(Instruction::decl_empty_var(var)),
        //             VarDeclKind::Let => self.instructions.push(Instruction::decl_empty_let(var)),
        //             VarDeclKind::Const => {
        //                 return Err(anyhow!("const in for...of is not supported yet"));
        //             }
        //         }
        //
        //         target_out = Some(OutputDataType::Var(var));
        //     }
        //     ForHead::Pat(pat) => {
        //         // Support bare identifier pattern: for (i of iterable)
        //         if let Some(ident) = pat.as_ident() {
        //             let var = self.alloc_var(ident.id.as_ref());
        //             target_out = Some(OutputDataType::Var(var));
        //         } else {
        //             return Err(anyhow!(
        //                 "Only simple identifier patterns are supported in for...of"
        //             ));
        //         }
        //     }
        //     ForHead::UsingDecl(_) => {
        //         return Err(anyhow!(
        //             "using declaration in for...of is not supported yet"
        //         ));
        //     }
        // }
        //
        // let iter_obj_dt: DataType = iter_obj_out.into();
        //
        // // Loop start
        // let start = self.instructions.len();
        //
        // // Call iterator.next()
        // let next_key = self.alloc_const(ConstValue::String("next".into()));
        // let result_out = self.alloc_reg_or_stack();
        // self.instructions
        //     .push(Instruction::call_member(iter_obj_dt, next_key, result_out));
        //
        // // Load result.done
        // let done_key = self.alloc_const(ConstValue::String("done".into()));
        // let done_out = self.alloc_reg_or_stack();
        // self.instructions
        //     .push(Instruction::load_member(result_out, done_key, done_out));
        //
        // // If done is true, exit loop
        // let jmp_pos = self.instructions.len();
        // self.instructions.push(Instruction::JmpRel(0));
        //
        // // Load result.value
        // let value_key = self.alloc_const(ConstValue::String("value".into()));
        // let value_out = self.alloc_reg_or_stack();
        // self.instructions
        //     .push(Instruction::load_member(result_out, value_key, value_out));
        //
        // // Assign to target
        // if let Some(target) = target_out {
        //     self.instructions
        //         .push(Instruction::move_(value_out, target));
        // }
        //
        // // Clean up temps (result and value no longer needed after assignment)
        // self.dealloc(value_out);
        // self.dealloc(result_out);
        //
        // // Body
        // self.compile_stmt(&f.body)?;
        //
        // // Jump back to start
        // self.instructions.push(Instruction::jmp(start));
        //
        // // Patch jump with condition "done == true"
        // if let Some(inst) = Test::Cond(done_out.into()).get(self.instructions.len() as JmpAddr) {
        //     self.instructions[jmp_pos] = inst;
        // }
        //
        // // Deallocate temps that live across iterations after loop exits
        // self.dealloc(done_out);
        // self.dealloc(iter_obj_out);

        // Ok(())

        todo!()
    }
}
