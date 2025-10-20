use crate::value::{MutObj, Obj, ObjectOrNull};
use common::{
    define_getter, define_setter, has_own_property, is_prototype_of, lookup_getter, lookup_setter,
    property_is_enumerable, to_locale_string, to_string, value_of,
};
use std::any::Any;
use yavashark_macro::inline_props;
use crate::object::constructor::ObjectConstructor;
use crate::object::prototype::common::get_own_property_descriptor;
use crate::realm::Realm;
use crate::{
    NativeFunction, ObjectHandle, Res, Value,
};
use crate::inline_props::InlineObject;
use crate::partial_init::{Initializer, Partial};

pub mod common;

#[inline_props(enumerable = false, configurable)]
#[derive(Debug, Default)]
pub struct Prototype {
    #[prop("__defineGetter__")]
    define_getter: Partial<ObjectHandle, DefineGetter>,

    #[prop("__defineSetter__")]
    define_setter: Partial<ObjectHandle, DefineSetter>,

    #[prop("__lookupGetter__")]
    lookup_getter: Partial<ObjectHandle, LookupGetter>,

    #[prop("__lookupSetter__")]
    lookup_setter: Partial<ObjectHandle, LookupSetter>,

    constructor: Partial<ObjectHandle, ObjectConstructor>,

    #[prop("hasOwnProperty")]
    has_own_property: Partial<ObjectHandle, HasOwnProperty>,

    #[prop("getOwnPropertyDescriptor")]
    get_own_property_descriptor: Partial<ObjectHandle, GetOwnPropertyDescriptor>,

    #[prop("isPrototypeOf")]
    is_prototype_of: Partial<ObjectHandle, IsPrototypeOf>,

    #[prop("propertyIsEnumerable")]
    property_is_enumerable: Partial<ObjectHandle, PropertyIsEnumerable>,

    #[prop("toLocaleString")]
    to_locale_string: Partial<ObjectHandle, ToLocaleString>,

    #[prop("toString")]
    to_string: Partial<ObjectHandle, ToString>,

    #[prop("valueOf")]
    value_of: Partial<ObjectHandle, ValueOf>,
}


#[macro_export]
macro_rules! proto {
    ($init_name:ident, $ident:ident, $name:literal, $len:literal) => {
        pub struct $init_name;

        impl Initializer<ObjectHandle> for $init_name {
            fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
                Ok(NativeFunction::with_len(
                    $name,
                    $ident,
                    realm,
                    $len,
                ))
            }
        }

    };
}

proto!(DefineGetter, define_getter, "__defineGetter__", 2);
proto!(DefineSetter, define_setter, "__defineSetter__", 2);
proto!(LookupGetter, lookup_getter, "__lookupGetter__", 1);
proto!(LookupSetter, lookup_setter, "__lookupSetter__", 1);
proto!(HasOwnProperty, has_own_property, "hasOwnProperty", 1);
proto!(GetOwnPropertyDescriptor, get_own_property_descriptor, "getOwnPropertyDescriptor", 1);
proto!(IsPrototypeOf, is_prototype_of, "isPrototypeOf", 1);
proto!(PropertyIsEnumerable, property_is_enumerable, "propertyIsEnumerable", 1);
proto!(ToLocaleString, to_locale_string, "toLocaleString", 0);
proto!(ToString, to_string, "toString", 0);
proto!(ValueOf, value_of, "valueOf", 0);


impl Prototype {
    #[must_use]
    pub fn new() -> InlineObject<Self> {
        InlineObject::with_proto(
            Prototype::default(),
            ObjectOrNull::Null,
        )
    }
}

pub struct GlobalObjectConstructor;

impl Initializer<ObjectHandle> for GlobalObjectConstructor {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        realm.intrinsics.obj
            .clone()
            .get("constructor", realm)?
            .to_object()
    }
}