mod execute;
mod instructions;
mod regs;
mod stack;
mod storage;
mod value_ext;

use crate::execute::Execute;
pub use regs::*;
pub use stack::*;
use yavashark_bytecode::data::DataSection;
use yavashark_bytecode::Instruction;
use yavashark_env::scope::{ParentOrGlobal, Scope};
use yavashark_env::{Error, Realm, Res, Value};

pub use yavashark_bytecode;

pub struct VM {
    regs: Registers,
    stack: Stack,

    pc: usize,
    code: Vec<Instruction>,
    data: DataSection,

    current_scope: Scope,

    acc: Value,

    realm: Realm,
}

impl VM {
    pub fn new(code: Vec<Instruction>, data: DataSection) -> anyhow::Result<Self> {
        let realm = Realm::new()?;

        Ok(Self {
            regs: Registers::new(),
            stack: Stack::new(),
            pc: 0,
            code,
            data,
            current_scope: Scope::new(&realm),
            acc: Value::Undefined,
            realm,
        })
    }

    #[must_use]
    pub fn with_realm(code: Vec<Instruction>, data: DataSection, realm: Realm) -> Self {
        Self {
            regs: Registers::new(),
            stack: Stack::new(),
            pc: 0,
            code,
            data,
            current_scope: Scope::new(&realm),
            acc: Value::Undefined,
            realm,
        }
    }

    pub fn get_realm(&mut self) -> &mut Realm {
        &mut self.realm
    }

    pub fn push_scope(&mut self) -> Res {
        self.current_scope = self.current_scope.child()?;

        Ok(())
    }

    pub fn pop_scope(&mut self) -> Res {
        let scope = self.current_scope.parent()?;

        if let ParentOrGlobal::Parent(p) = scope {
            self.current_scope = p.into();
        } else {
            return Err(Error::new("No parent scope"));
        }

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
            let instr = self.code[self.pc];
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
        let realm = Realm::new().unwrap();

        let mut vm = VM {
            regs: Registers::new(),
            stack: Stack::new(),
            pc: 0,
            code: vec![
                Instruction::LdaAcc(0),
                Instruction::PushAcc,
                Instruction::LoadEnv(0),
                Instruction::LoadMemberAcc(1),
                Instruction::CallAcc(1),
                Instruction::LdaAcc(1),
                Instruction::JmpIfNotAccRel(7),
                Instruction::LdaAcc(2),
                Instruction::PushAcc,
                Instruction::LoadEnv(2),
                Instruction::LoadMemberAcc(3),
                Instruction::CallAcc(1),
                Instruction::JmpRel(6),
                Instruction::LdaAcc(3),
                Instruction::PushAcc,
                Instruction::LoadEnv(4),
                Instruction::LoadMemberAcc(5),
                Instruction::CallAcc(1),
            ],
            data: DataSection {
                var_names: vec![
                    "console".to_string(),
                    "log".to_string(),
                    "console".to_string(),
                    "log".to_string(),
                    "console".to_string(),
                    "log".to_string(),
                ],
                constants: vec![
                    ConstValue::String("Hello, World!".into()),
                    ConstValue::Boolean(true),
                    ConstValue::String("True".into()),
                    ConstValue::String("False".into()),
                ],
            },
            current_scope: Scope::new(&realm),
            acc: Value::Undefined,
            realm,
        };

        vm.run().unwrap();
    }
}
