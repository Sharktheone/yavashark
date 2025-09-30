use crate::error_obj::ErrorObj;
use crate::{Error, NativeConstructor, Object, ObjectHandle, Res, Value, Variable};
use crate::value::Obj;

macro_rules! error {
    ($name:ident, $create:ident, $get:ident) => {
        pub fn $get(error: Value, error_proto: Value) -> Res<ObjectHandle> {
            let proto = Object::with_proto(error);

            proto.define_property("name".into(), stringify!($name).into())?;

            let constr = NativeConstructor::with_proto(
                stringify!($name).into(),
                |args, realm| {
                    let msg = args
                        .first()
                        .map_or(Result::<String, Error>::Ok(String::new()), |x| {
                            Ok(x.to_string(realm)?.to_string())
                        })?;

                    let obj = ErrorObj::raw(Error::$create(msg), realm);

                    Ok(obj.into_value())
                },
                error_proto.clone(),
                error_proto,
            );

            constr.define_variable(
                "prototype".into(),
                Variable::new_read_only(proto.clone().into()),
            )?;
            constr.define_variable("name".into(), Variable::config(stringify!($name).into()))?;

            constr.define_variable("length".into(), Variable::config(1.into()))?;

            proto.define_variable("constructor".into(), Variable::write_config(constr.into()))?;

            Ok(proto.into())
        }
    };
}

error!(TypeError, ty_error, get_type_error);
error!(ReferenceError, reference_error, get_reference_error);
error!(RangeError, range_error, get_range_error);
error!(SyntaxError, syn_error, get_syntax_error);
error!(EvalError, eval_error, get_eval_error);
error!(URIError, uri_error, get_uri_error);
error!(AggregateError, aggregate_error, get_aggregate_error);
