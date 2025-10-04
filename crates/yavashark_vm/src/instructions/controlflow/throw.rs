use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::error::Error;
use yavashark_env::{ControlFlow, ControlResult};

pub fn throw_acc(vm: &impl VM) -> ControlResult {
    let acc = vm.acc();
    Err(ControlFlow::Error(Error::throw(acc)))
}

pub fn throw_reg(reg: Reg, vm: &impl VM) -> ControlResult {
    let value = vm.get_register(reg)?;
    Err(ControlFlow::Error(Error::throw(value)))
}

pub fn throw(var: VarName, vm: &mut impl VM) -> ControlResult {
    let value = vm.get_variable(var)?;
    Err(ControlFlow::Error(Error::throw(value)))
}
