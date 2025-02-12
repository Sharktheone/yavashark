use crate::VM;
use yavashark_env::Res;

pub fn push_scope(vm: &mut VM) -> Res {
    vm.push_scope()?;

    Ok(())
}

pub fn pop_scope(vm: &mut VM) -> Res {
    vm.pop_scope()?;

    Ok(())
}
