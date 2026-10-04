use crate::error_obj::ErrorObj;
use crate::realm::Intrinsic;
use crate::value::Obj;
use crate::{Error, NativeConstructor, Object, ObjectHandle, Realm, Res, Value, Variable};

macro_rules! error {
    ($name:ident, $create:ident, $get:ident, $message_index:expr) => {
        pub fn $get(realm: &mut Realm) -> Res<ObjectHandle> {
            let error = realm.intrinsics.clone_public().error.get(realm)?.clone();

            let error_proto = error
                .resolve_property("constructor", realm)
                .unwrap_or(Value::Undefined.into())
                .unwrap_or(Value::Undefined.into())
                .to_object()?;

            let proto = Object::with_proto(error);

            proto.define_property_attributes(
                "name".into(),
                Variable::new_with_attributes(stringify!($name).into(), true, false, true),
                realm,
            )?;

            proto.define_property_attributes(
                "message".into(),
                Variable::new_with_attributes("".into(), true, false, true),
                realm,
            )?;

            let constr = NativeConstructor::special_with_proto(
                stringify!($name).into(),
                |args, realm| {
                    let message = args
                        .get($message_index)
                        .filter(|value| !value.is_undefined())
                        .map(|value| {
                            if value.is_symbol() {
                                return Err(Error::ty("Cannot convert a Symbol value to a string"));
                            }

                            value.to_string(realm)
                        })
                        .transpose()?;

                    let error = Error::$create(
                        message
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_default(),
                    );
                    let obj = ErrorObj::with_message(error, message, realm)?;

                    Ok(obj.into_object())
                },
                error_proto.clone(),
                error_proto,
            );

            constr.define_property_attributes(
                "prototype".into(),
                Variable::new_read_only(proto.clone().into()),
                realm,
            )?;
            constr.define_property_attributes(
                "name".into(),
                Variable::config(stringify!($name).into()),
                realm,
            )?;

            constr.define_property_attributes(
                "length".into(),
                Variable::config(1.into()),
                realm,
            )?;

            proto.define_property_attributes(
                "constructor".into(),
                Variable::write_config(constr.into()),
                realm,
            )?;

            Ok(proto.into())
        }

        pub struct $name;

        impl Intrinsic for $name {
            fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
                $get(realm)
            }

            fn get_intrinsic(realm: &mut Realm) -> Res<ObjectHandle> {
                Ok(realm.intrinsics.clone_public().$create.get(realm)?.clone())
            }

            fn get_global(realm: &mut Realm) -> Res<ObjectHandle> {
                Self::get_intrinsic(realm)?
                    .get("constructor", realm)?
                    .to_object()
            }
        }
    };
}

error!(TypeError, ty_error, get_type_error, 0);
error!(ReferenceError, reference_error, get_reference_error, 0);
error!(RangeError, range_error, get_range_error, 0);
error!(SyntaxError, syn_error, get_syntax_error, 0);
error!(EvalError, eval_error, get_eval_error, 0);
error!(URIError, uri_error, get_uri_error, 0);
error!(AggregateError, aggregate_error, get_aggregate_error, 1);
error!(SuppressedError, suppressed_error, get_supressed_error, 2);
