use crate::data::{Data, OutputData};
use crate::VM;
use yavashark_env::utils::ValueIterator;
use yavashark_env::{ControlFlow, ControlResult, Res, Value};

pub fn call(func: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let func = func.get(vm)?;

    let args = vm.get_call_args();
    let this = vm.get_this()?;

    let ret = func.call(vm.get_realm(), args, this)?;

    output.set(ret, vm)
}

pub fn call_no_output(func: impl Data, vm: &mut impl VM) -> Res {
    let func = func.get(vm)?;

    let args = vm.get_call_args();
    let this = vm.get_this()?;

    func.call(vm.get_realm(), args, this)?;

    Ok(())
}

pub fn call_member(
    obj: impl Data,
    member: impl Data,
    output: impl OutputData,
    vm: &mut impl VM,
) -> Res {
    let obj = obj.get(vm)?;
    let member = member.get(vm)?;

    let args = vm.get_call_args();

    let ret = obj.call_method(&member, vm.get_realm(), args)?;

    output.set(ret, vm)
}

pub fn call_member_no_output(
    obj: impl Data,
    member: impl Data,
    vm: &mut impl VM,
) -> Res {
    let obj = obj.get(vm)?;
    let member = member.get(vm)?;

    let args = vm.get_call_args();

    obj.call_method(&member, vm.get_realm(), args)?;
    
    Ok(())
}

pub fn construct(func: impl Data, output: impl OutputData, vm: &mut impl VM) -> ControlResult {
    let func = func.get(vm)?;


    let Value::Object(constructor) = func.copy() else {
        return Err(ControlFlow::error_type(format!(
            "{:?} is not a constructor",
            func
        )));
    };

    let args = vm.get_call_args();

    let ret = constructor.construct(vm.get_realm(), args)?;

    output.set(ret, vm)?;
    
    Ok(())
}

pub fn construct_no_output(func: impl Data, vm: &mut impl VM) -> ControlResult {
    let func = func.get(vm)?;

    let Value::Object(constructor) = func.copy() else {
        return Err(ControlFlow::error_type(format!(
            "{:?} is not a constructor",
            func
        )));
    };

    let args = vm.get_call_args();

    constructor.construct(vm.get_realm(), args)?;

    Ok(())
}

pub fn call_super(output: impl OutputData, vm: &mut impl VM) -> Res {
    let class = vm.get_scope().this()?;
    let realm = vm.get_realm();

    let proto = class.prototype(realm)?;
    let sup = proto.prototype(realm)?;

    let constructor = sup.as_object()?.constructor()?;

    let constructor = constructor.resolve(proto.copy(), realm)?;

    let constructor = constructor.as_object()?;
    
    let args = vm.get_call_args();
    let ret = constructor
        .construct(vm.get_realm(), args)?;

    output.set(ret, vm)
}

pub fn call_super_no_output(vm: &mut impl VM) -> Res {
    let class = vm.get_scope().this()?;
    let realm = vm.get_realm();

    let proto = class.prototype(realm)?;
    let sup = proto.prototype(realm)?;

    let constructor = sup.as_object()?.constructor()?;

    let constructor = constructor.resolve(proto.copy(), realm)?;

    let constructor = constructor.as_object()?;
    
    let args = vm.get_call_args();
    
    constructor
        .construct(vm.get_realm(), args)?;

    Ok(())
}

pub fn push_call(arg: impl Data, vm: &mut impl VM) -> Res {
    let arg = arg.get(vm)?;

    vm.push_call_arg(arg);

    Ok(())
}

pub fn spread_call(args: impl Data, vm: &mut impl VM) -> Res {
    let args = args.get(vm)?;

    let iter = ValueIterator::new(&args, vm.get_realm())?;

    while let Some(value) = iter.next(vm.get_realm())? {
        vm.push_call_arg(value);
    }

    Ok(())
}
