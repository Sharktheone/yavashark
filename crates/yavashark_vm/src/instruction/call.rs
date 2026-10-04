use crate::VM;
use crate::data::{Data, OutputData};
use crate::instruction::get_private_member;
use yavashark_env::utils::ValueIterator;
use yavashark_env::{ControlFlow, ControlResult, Error, Res, Value};

pub fn call(func: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let ret = call_value(func, vm)?;

    output.set(ret, vm)
}

pub fn call_no_output(func: impl Data, vm: &mut impl VM) -> Res {
    call_value(func, vm)?;

    Ok(())
}

fn call_value(reference: impl Data, vm: &mut impl VM) -> yavashark_env::ValueResult {
    let direct_eval = matches!(
        reference.data_type(),
        yavashark_bytecode::data::DataType::Var(name) if vm.var_name(name.0) == Some("eval")
    );

    let function = reference.get(vm)?;
    let args = vm.get_call_args();

    if direct_eval {
        let is_eval = matches!(
            &function,
            Value::Object(function) if vm.get_realm_ref().intrinsics.eval.as_ref() == Some(function)
        );

        if is_eval {
            let Some(value) = args.first() else {
                return Ok(Value::Undefined);
            };

            let Value::String(source) = value else {
                return Ok(value.copy());
            };

            let mut scope = vm.get_scope().clone();

            return vm
                .get_realm()
                .eval_in_scope(&source.as_str_lossy(), &mut scope);
        }
    }

    function.call(vm.get_realm(), args, Value::Undefined)
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

pub fn call_member_no_output(obj: impl Data, member: impl Data, vm: &mut impl VM) -> Res {
    let obj = obj.get(vm)?;
    let member = member.get(vm)?;

    let args = vm.get_call_args();

    obj.call_method(&member, vm.get_realm(), args)?;

    Ok(())
}

pub fn call_private_member(
    obj: impl Data,
    member: impl Data,
    output: impl OutputData,
    vm: &mut impl VM,
) -> Res {
    let base = obj.get(vm)?;

    let right = member.get(vm)?;

    let Value::String(name) = right else {
        return Err(Error::ty("Private member name must be a string"));
    };

    let res = get_private_member(vm.get_realm(), base, &name.as_str_lossy())?;

    let args = vm.get_call_args();

    let this = res.1.unwrap_or(vm.get_this()?);

    let res = res.0.call(vm.get_realm(), args, this)?;

    output.set(res, vm)?;

    Ok(())
}

pub fn call_private_member_no_output(obj: impl Data, member: impl Data, vm: &mut impl VM) -> Res {
    let base = obj.get(vm)?;

    let right = member.get(vm)?;

    let Value::String(name) = right else {
        return Err(Error::ty("Private member name must be a string"));
    };

    let res = get_private_member(vm.get_realm(), base, &name.as_str_lossy())?;

    let args = vm.get_call_args();

    let this = res.1.unwrap_or(vm.get_this()?);

    res.0.call(vm.get_realm(), args, this)?;

    Ok(())
}
pub fn construct(func: impl Data, output: impl OutputData, vm: &mut impl VM) -> ControlResult {
    let func = func.get(vm)?;

    let Value::Object(constructor) = func.copy() else {
        return Err(ControlFlow::error_type(format!(
            "{func:?} is not a constructor",
        )));
    };

    let args = vm.get_call_args();

    let ret = constructor.construct(args, vm.get_realm())?;

    output.set(ret.into(), vm)?;

    Ok(())
}

pub fn construct_no_output(func: impl Data, vm: &mut impl VM) -> ControlResult {
    let func = func.get(vm)?;

    let Value::Object(constructor) = func.copy() else {
        return Err(ControlFlow::error_type(format!(
            "{func:?} is not a constructor",
        )));
    };

    let args = vm.get_call_args();

    constructor.construct(args, vm.get_realm())?;

    Ok(())
}

pub fn call_super(output: impl OutputData, vm: &mut impl VM) -> Res {
    let class = vm.get_scope().this()?;
    let realm = vm.get_realm();

    let proto = class.prototype(realm)?.to_object()?;

    let sup = proto.prototype(realm)?;

    let constructor = sup.to_object()?.get("constructor", realm)?;

    let constructor = constructor.as_object()?;

    let args = vm.get_call_args();
    let ret = constructor.construct(args, vm.get_realm())?;

    output.set(ret.into(), vm)
}

pub fn call_super_no_output(vm: &mut impl VM) -> Res {
    let class = vm.get_scope().this()?;
    let realm = vm.get_realm();

    let proto = class.prototype(realm)?.to_object()?;

    let sup = proto.prototype(realm)?;

    let constructor = sup.to_object()?.get("constructor", realm)?;

    let constructor = constructor.as_object()?;

    let args = vm.get_call_args();

    constructor.construct(args, vm.get_realm())?;

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
