use crate::{Compiler, Res};
use swc_ecma_ast::TryStmt;
use yavashark_bytecode::control::TryBlock;
use yavashark_bytecode::instructions::Instruction;
use yavashark_bytecode::{JmpAddr, JmpOffset};

impl Compiler {
    pub fn compile_try(&mut self, s: &TryStmt) -> Res {
        let try_block = self.new_try_block();

        self.instructions.push(Instruction::enter_try(try_block));

        self.compile_block(&s.block)?;

        self.instructions.push(Instruction::leave_try());

        let mut catch_addr = None;

        if let Some(catch) = &s.handler {
            catch_addr = Some(self.instructions.len() as JmpAddr);
            //TODO: compile param

            self.compile_block(&catch.body)?;

            self.instructions.push(Instruction::leave_try());
        }

        let mut finally_addr = None;

        if let Some(finally) = &s.finalizer {
            finally_addr = Some(self.instructions.len() as JmpAddr);
            self.compile_block(finally)?;
        }

        let block = TryBlock {
            exit: self.instructions.len() as JmpAddr,
            catch: catch_addr,
            catch_pat: None,
            finally: finally_addr,
        };

        self.set_try(try_block, block);

        Ok(())
    }
}
