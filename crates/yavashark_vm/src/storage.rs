use crate::VM;
use yavashark_bytecode::{ConstIdx, Reg, VarName};
use yavashark_env::{Res, Value, Result};
use yavashark_env::value::Error;

#[allow(unused)]
impl VM {
    pub fn get_variable(&self, name: VarName) -> Result<Option<Value>> {
        
        let Some(name) = self.var_name(name) else {
            return Ok(None);
        };
        
        self.current_scope.resolve(name)
    }

    pub fn var_name(&self, name: VarName) -> Option<&str> {
        self.data.var_names.get(name as usize).map(|s| s.as_str())
    }

    pub fn get_register(&self, reg: Reg) -> Option<Value> {
        self.regs.get(reg)
    }

    pub fn set_variable(&mut self, name: VarName, value: Value) -> Res {
        let name = self.var_name(name).ok_or(Error::reference("Invalid variable name"))?;
        self.current_scope.declare_var(name.into(), value)
    }

    pub fn set_register(&mut self, reg: Reg, value: Value) -> Res {
        self.regs.set(reg, value)
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    pub fn acc(&self) -> Value {
        self.acc.clone()
    }

    pub fn set_acc(&mut self, value: Value) {
        self.acc = value
    }

    pub fn set_accb(&mut self, value: bool) {
        self.acc = Value::Boolean(value)
    }

    pub fn get_this(&self) -> Result<Value> {
        self.current_scope.this()
    }

    pub fn get_constant(&self, const_idx: ConstIdx) -> Option<Value> {
        todo!()
    }

    pub fn get_stack(&self, idx: u32) -> Option<Value> {
        self.stack.get(idx as usize).cloned()
    }
    
    
    pub fn get_args(&mut self, num: u16) -> Vec<Value> {
        self.stack.pop_n(num as usize)
    }
}
