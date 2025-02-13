use std::cell::{Ref, RefMut};
use std::ops::{Deref, DerefMut};
use yavashark_value::{MutObj, Obj};
use crate::error::{ErrorObj, MutableErrorObj};
use crate::{Realm, Value, Object, NativeConstructor, Error, Result, ObjectHandle};



macro_rules! error {
    ($name:ident, $create:ident, $get:ident) => {
    pub fn $get(error: Value, func: Value) -> Result<ObjectHandle> {
        let proto = Object::with_proto(error);

        proto.define_property("name".into(), stringify!($name).into())?;
        proto.define_property("message".into(), "".into())?;
        proto.define_property("stack".into(), "".into())?;



        let constr = NativeConstructor::with_proto(stringify!($name).into(), |args, realm| {
            let msg = args.first().map_or(Ok(String::new()), |x| x.to_string(realm))?;

            let obj = ErrorObj::raw(Error::$create(msg), realm);

            Ok(obj.into_value())
        }, func.clone(), func);

        constr.define_property("prototype".into(), proto.clone().into())?;
        constr.define_property("name".into(), stringify!($name).into())?;


        proto.define_property("constructor".into(), constr.into())?;

        Ok(proto.into())
    }
        
        
    };
}

error!(TypeError, ty_error, get_type_error);
error!(ReferenceError, reference_error, get_reference_error);
error!(RangeError, range_error, get_range_error);
error!(SyntaxError, syn_error, get_syntax_error);
