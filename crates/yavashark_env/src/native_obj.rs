#![allow(unused)]

use std::cell::RefCell;
use std::fmt::Debug;
use crate::{MutObject, ObjectHandle, Realm, Res, Value};
use crate::conversion::TryIntoValue;
use crate::inline_props::{InlineObject, PropertiesHook};
use crate::realm::Intrinsic;
use crate::value::IntoValue;

// TODO: maybe this is possible at some point - sigh >'_'<
// default impl<T: Intrinsic + PropertiesHook + Debug + 'static> TryIntoValue for T {
//     fn try_into_value(self, realm: &mut Realm) -> Res<Value> {
//         let proto = T::get_intrinsic(realm)?;
//
//         let obj = InlineObject::with_proto(self, proto);
//
//         Ok(obj.into_value())
//     }
// }
