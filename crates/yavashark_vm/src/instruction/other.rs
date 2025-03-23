use yavashark_env::Res;
use crate::data::{Data, OutputData};
use crate::VM;

pub fn nullish_coalescing(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;

    if left.is_nullish() {
        output.set(right, vm)?;
    } else {
        output.set(left, vm)?;
    }

    Ok(())
}

pub fn in_(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;

    let result = left.contains_key(&right)?.into();

    output.set(result, vm)
}

pub fn instance_of(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;

    let result = left.instance_of(&right, vm.get_realm())?.into();

    output.set(result, vm)
}
