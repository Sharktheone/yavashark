use yavashark_codegen::ByteCodegen;
use yavashark_env::optimizer::OptimFunction;
use yavashark_env::{
    Error, NativeFunction, ObjectHandle, Realm, Res, Value,
};
use yavashark_env::conversion::downcast_obj;
use yavashark_interpreter::function::OptimizedJSFunction;
use yavashark_vm::function_code::OldBytecodeFunction;
use yavashark_vm::yavashark_bytecode::data::DataSection;

pub fn define_optimizer(realm: &Realm) -> Res {
    let optimizer = get_optimizer(realm);

    realm
        .global
        .define_variable("optimize".into(), optimizer.into())?;

    Ok(())
}

fn get_optimizer(realm: &Realm) -> ObjectHandle {
    NativeFunction::new(
        "optimizer",
        |args, _, _| {
            let Some(func) = args.first() else {
                return Err(Error::ty(
                    "optimizer expects a function as its first argument",
                ));
            };

            let func = downcast_obj::<OptimFunction>(func.copy())?;
            println!("Optimizing function: {}", func.raw.name);

            let Some(code) = &func.raw.block else {
                return Ok(Value::Undefined);
            };

            let mut code = code.borrow_mut();
            let func = {
                let code_any = code.function_any();

                let Some(opt_code) = code_any.downcast_ref::<OptimizedJSFunction>() else {
                    // Already optimized
                    return Ok(Value::Undefined);
                };

                let bc = match ByteCodegen::compile(&opt_code.block.stmts) {
                    Ok(bc) => bc,
                    Err(e) => {
                        return Err(Error::syn_error(format!("Failed to compile code: {e:?}")));
                    }
                };

                let data = DataSection::new(bc.variables, Vec::new(), bc.literals, Vec::new());

                OldBytecodeFunction {
                    instructions: bc.instructions,
                    ds: data,
                }
            };

            *code = Box::new(func);

            Ok(Value::Undefined)
        },
        realm,
    )
}
