use crate::{Compiler, Res};
use std::path::Component;
use anyhow::anyhow;
use swc_ecma_ast::{ArrayPat, Pat};
use yavashark_bytecode::data::Data;
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_pat(&mut self, pat: &Pat, source: impl Data) -> Res {
        match pat {
            Pat::Array(array) => self.compile_array_pat(array, source)?,
            Pat::Ident(ident) => {
                let name = self.alloc_var(ident.as_ref());

                self.instructions.push(Instruction::decl_let(source, name));
            },
            Pat::Invalid(invalid) => {
                Err(anyhow!("Invalid pattern: {:?}", invalid))?
            },
            _ => todo!()
        }

        Ok(())
    }

    pub fn compile_array_pat(&mut self, array: &ArrayPat, source: impl Data) -> Res {
        let iter = self.alloc_reg_or_stack();

        self.instructions.push(Instruction::push_iter(source, iter));

        let out = self.alloc_reg_or_stack();
        for (i, elem) in array.elems.iter().enumerate() {
            self.instructions.push(Instruction::iter_next(iter, out));

            if let Some(elem) = elem {
                self.compile_pat(elem, out)?;
            } else {
                self.instructions.push(Instruction::iter_next_no_output(iter))
            }
        }

        self.dealloc(iter);
        self.dealloc(out);



        Ok(())

    }
}
