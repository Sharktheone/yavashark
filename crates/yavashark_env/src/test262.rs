use crate::{Context, NativeFunction, ObjectHandle, Value, ValueResult};
use yavashark_macro::{object, properties};
use yavashark_value::Error;

pub fn print(ctx: &mut Context) -> ObjectHandle {
    NativeFunction::new(
        "print",
        |args, _, _| {
            let Some(first) = args.first() else {
                return Err(Error::ty("expected at least one argument"));
            };

            println!("{first:?}");

            Ok(Value::Undefined)
        },
        ctx,
    )
}

#[object(direct(abstract_module_source(AbstractModuleSource)))]
#[derive(Debug)]
struct Test262 {}


#[properties]
impl Test262 {
    #[prop(createRealm)]
    fn create_realm(&mut self, args: Vec<Value>, ctx: &mut Context) -> ValueResult {
        Ok(Value::Undefined)
    }


    #[prop(detachArrayBuffer)]
    fn detach_array_buffer(&mut self, args: Vec<Value>, ctx: &mut Context) -> ValueResult {
        Ok(Value::Undefined)
    }

    #[prop(evalScript)]
    fn eval_script(&mut self, args: Vec<Value>, ctx: &mut Context) -> ValueResult {
        Ok(Value::Undefined)
    }

    fn gc(&mut self, args: Vec<Value>, ctx: &mut Context) -> ValueResult {
        Ok(Value::Undefined)
    }

    #[prop(IsHTMLDDA)]
    fn is_htmldda(&self, args: Vec<Value>, ctx: &mut Context) -> ValueResult {
        Ok(Value::Undefined)
    }
}