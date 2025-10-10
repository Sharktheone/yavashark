use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::ControlResult;

pub fn in_(lhs: VarName, rhs: VarName, vm: &mut impl VM) -> ControlResult {
    let lhs = vm.get_variable(lhs)?;
    let rhs = vm.get_variable(rhs)?;

    let acc = rhs.contains_key(&lhs, vm.get_realm())?.into();

    vm.set_acc(acc);

    Ok(())
}

pub fn in_acc(reg: Reg, vm: &mut impl VM) -> ControlResult {
    let rhs = vm.get_register(reg)?;
    let lhs = vm.acc();

    let acc = rhs.contains_key(&lhs, vm.get_realm())?.into();

    vm.set_acc(acc);

    Ok(())
}

pub fn in_reg(rhs: Reg, lhs: Reg, vm: &mut impl VM) -> ControlResult {
    let rhs = vm.get_register(rhs)?;
    let lhs = vm.get_register(lhs)?;

    let acc = rhs.contains_key(&lhs, vm.get_realm())?.into();

    vm.set_acc(acc);

    Ok(())
}
