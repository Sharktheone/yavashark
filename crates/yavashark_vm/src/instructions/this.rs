use crate::VM;
use yavashark_bytecode::Reg;
use yavashark_env::Res;

pub fn this_acc(vm: &mut VM) -> Res {
    let this = vm.get_this()?;
    vm.set_acc(this);

    Ok(())
}

pub fn this_reg(reg: Reg, vm: &mut VM) -> Res {
    let this = vm.get_this()?;

    vm.set_register(reg, this)?;

    Ok(())
}
