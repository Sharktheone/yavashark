use crate::vm::owned::OwnedVM;
use yavashark_bytecode::{ConstIdx, Reg, VarName};
use yavashark_env::value::Error;
use yavashark_env::{Res, Value};

#[allow(unused)]
impl OwnedVM {
    pub fn get_variable(&mut self, name: VarName) -> Res<Value> {
        let Some(name) = self.data.var_names.get(name as usize) else {
            return Err(Error::reference("Invalid variable name"));
        };

        self.current_scope
            .resolve(name)?
            .ok_or(Error::reference("Variable not found"))
    }

    #[must_use]
    pub fn var_name(&self, name: VarName) -> Option<&str> {
        self.data
            .var_names
            .get(name as usize)
            .map(std::string::String::as_str)
    }

    pub fn get_register(&self, reg: Reg) -> Res<Value> {
        self.regs
            .get(reg)
            .ok_or(Error::reference("Invalid register"))
    }

    pub fn set_variable(&mut self, name: VarName, value: Value) -> Res {
        let name = self
            .var_name(name)
            .ok_or(Error::reference("Invalid variable name"))?;
        self.current_scope.declare_var(name.into(), value)
    }

    pub fn set_register(&mut self, reg: Reg, value: Value) -> Res {
        self.regs.set(reg, value)
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    #[must_use]
    pub fn acc(&self) -> Value {
        self.acc.clone()
    }

    pub fn set_acc(&mut self, value: Value) {
        self.acc = value;
    }

    pub fn set_accb(&mut self, value: bool) {
        self.acc = Value::Boolean(value);
    }

    pub fn get_this(&self) -> Res<Value> {
        self.current_scope.this()
    }

    pub fn get_constant(&self, const_idx: ConstIdx) -> Res<Value> {
        let val = self
            .data
            .constants
            .get(const_idx as usize)
            .ok_or(Error::reference("Invalid constant index"))?;

        val.clone().into_value(&self.realm)
    }

    #[must_use]
    pub fn get_stack(&self, idx: u32) -> Option<Value> {
        self.stack.get(idx as usize).cloned()
    }

    pub fn set_stack(&mut self, idx: u32, value: Value) -> Res {
        self.stack.set(idx as usize, value);

        Ok(())
    }

    pub fn get_args(&mut self, num: u16) -> Vec<Value> {
        self.stack.pop_n(num as usize)
    }
}
