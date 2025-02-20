use crate::VM;
use yavashark_env::Res;

pub fn push_scope(vm: &mut impl VM) -> Res {
    vm.push_scope()?;

    Ok(())
}

pub fn pop_scope(vm: &mut impl VM) -> Res {
    vm.pop_scope()?;

    Ok(())
}
