use yavashark_bytecode::Reg;
use yavashark_env::value::Error;
use yavashark_env::{Res, Value};

pub const NUM_REGS: usize = 32;

pub struct Registers {
    regs: [Value; NUM_REGS],
}

impl Registers {
    pub fn new() -> Self {
        Self {
            regs: [const { Value::Undefined }; NUM_REGS],
        }
    }

    pub fn get(&self, reg: Reg) -> Option<Value> {
        self.regs.get(reg as usize).cloned()
    }

    pub fn set(&mut self, reg: Reg, value: Value) -> Res {
        self.regs
            .get_mut(reg as usize)
            .map(|r| *r = value)
            .ok_or(Error::new("Invalid register"))
    }
}
