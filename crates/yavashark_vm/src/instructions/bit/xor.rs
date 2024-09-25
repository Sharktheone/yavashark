use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::Res;

pub fn bitwise_xor(lhs: VarName, rhs: VarName, vm: &mut VM) -> Res {
    let lhs = vm.get_variable(lhs)?;
    let rhs = vm.get_variable(rhs)?;

    vm.set_acc(lhs ^ rhs);

    Ok(())
}

pub fn bitwise_xor_acc(reg: Reg, vm: &mut VM) -> Res {
    let rhs = vm.get_register(reg)?;
    let lhs = vm.acc();

    vm.set_acc(lhs ^ rhs);

    Ok(())
}

pub fn bitwise_xor_reg(rhs: Reg, lhs: Reg, vm: &mut VM) -> Res {
    let rhs = vm.get_register(rhs)?;
    let lhs = vm.get_register(lhs)?;

    vm.set_acc(lhs ^ rhs);

    Ok(())
}
