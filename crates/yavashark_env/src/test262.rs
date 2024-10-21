use yavashark_value::Error;
use crate::{Context, NativeFunction, ObjectHandle, Value};

pub fn print(ctx: &mut Context) -> ObjectHandle {
    NativeFunction::new("print", |args, _, _| {
        
        
        let Some(first) = args.first() else {
            return Err(Error::ty("expected at least one argument"));
        };
        
        
        println!("{first:?}");
        
        
        Ok(Value::Undefined)
    }, ctx)
}