mod execute;
mod instructions;
mod regs;
mod stack;
mod storage;
mod value_ext;
mod data;

use crate::data::DataSection;
pub use regs::*;
pub use stack::*;
use yavashark_bytecode::Instruction;
use yavashark_env::scope::Scope;
use yavashark_env::{Context, Error, Res, Value};
use crate::execute::Execute;

pub struct VM {
    regs: Registers,
    stack: Stack,

    pc: usize,
    code: Vec<Instruction>,
    data: DataSection,

    current_scope: Scope,

    acc: Value,

    ctx: Context,
}

impl VM {
    pub fn get_context(&mut self) -> &mut Context {
        &mut self.ctx
    }

    pub fn push_scope(&mut self) -> Res {
        self.current_scope = self.current_scope.child()?;

        Ok(())
    }

    pub fn pop_scope(&mut self) -> Res {
        self.current_scope = self.current_scope.parent()?.ok_or(Error::new("No parent scope"))?;

        Ok(())
    }

    pub fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }

    pub fn offset_pc(&mut self, offset: isize) {
        // pc won't be above isize::MAX, since this is `Vec`'s length limit
        self.pc = (self.pc as isize + offset) as usize;
    }
    
    pub fn run(&mut self) -> Res {
        while self.pc < self.code.len() {
            let instr = self.code[self.pc].clone();
            self.pc += 1;

            instr.execute(self)?;
        }
        
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use yavashark_bytecode::{ConstValue, Instruction};

    #[test]
    fn test_vm() {
        let ctx = Context::new().unwrap();

        let mut vm = VM {
            regs: Registers::new(),
            stack: Stack::new(),
            pc: 0,
            code: vec![Instruction::LdaAcc(0), Instruction::PushAcc, Instruction::LoadEnv(0), Instruction::LoadMemberAcc(1), Instruction::CallAcc(1), Instruction::LdaAcc(1), Instruction::JmpIfNotAccRel(7), Instruction::LdaAcc(2), Instruction::PushAcc, Instruction::LoadEnv(2), Instruction::LoadMemberAcc(3), Instruction::CallAcc(1), Instruction::JmpRel(6), Instruction::LdaAcc(3), Instruction::PushAcc, Instruction::LoadEnv(4), Instruction::LoadMemberAcc(5), Instruction::CallAcc(1)],
            data: DataSection {
                var_names: vec!["console".to_string(), "log".to_string(), "console".to_string(), "log".to_string(), "console".to_string(), "log".to_string()],
                constants: vec![ConstValue::String("Hello, World!".into()), ConstValue::Boolean(true), ConstValue::String("True".into()), ConstValue::String("False".into())],
            },
            current_scope: Scope::new(&ctx),
            acc: Value::Undefined,
            ctx,
        };
        
        vm.run().unwrap()
    }
}