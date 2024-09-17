use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::Res;

pub fn instance_of(lhs: VarName, rhs: VarName, vm: &mut VM) -> Res {
    let lhs = vm.get_variable(lhs);
    let rhs = vm.get_variable(rhs);

    let instance_of = lhs.instance_of(&rhs, vm.get_context())?;

    vm.set_acc(instance_of.into());

    Ok(())
}

pub fn instance_of_acc(reg: Reg, vm: &mut VM) -> Res {
    let rhs = vm.get_register(reg);
    let lhs = vm.acc();

    let instance_of = lhs.instance_of(&rhs, vm.get_context())?;

    vm.set_acc(instance_of.into());

    Ok(())
}

pub fn instance_of_reg(rhs: Reg, lhs: Reg, vm: &mut VM) -> Res {
    let rhs = vm.get_register(rhs);
    let lhs = vm.get_register(lhs);

    let instance_of = lhs.instance_of(&rhs, vm.get_context())?;

    vm.set_acc(instance_of.into());

    Ok(())
}
