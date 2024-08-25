use yavashark_bytecode::{Reg, VarName};
use yavashark_env::{ControlFlow, ControlResult};
use yavashark_env::value::Error;
use crate::VM;

pub fn throw_acc(vm: &mut VM) -> ControlResult {
    let acc = vm.acc();
    Err(ControlFlow::Error(Error::throw(acc)))
}

pub fn throw_reg(reg: Reg, vm: &mut VM) -> ControlResult {
    let value = vm.get_register(reg);
    Err(ControlFlow::Error(Error::throw(value)))
}

pub fn throw(var: VarName, vm: &mut VM) -> ControlResult {
    let value = vm.get_variable(var);
    Err(ControlFlow::Error(Error::throw(value)))
}


