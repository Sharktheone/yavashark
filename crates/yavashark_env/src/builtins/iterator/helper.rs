// 27.1.2 The %IteratorHelperPrototype% Object
//
// This module sets up the shared prototype for all iterator helpers.

use super::Iterator;
use crate::value::IntoValue;
use crate::{Error, ObjectHandle, Realm, Res, Symbol, Value, ValueResult};
use yavashark_macro::props;

/// Trait for iterator helper implementations
/// Each iterator helper must implement this to provide next/return behavior
pub trait IteratorHelperImpl: std::fmt::Debug + 'static {
    fn next_impl(&self, realm: &mut Realm) -> Res<ObjectHandle>;
    fn return_impl(&self, realm: &mut Realm) -> Res<ObjectHandle>;
}

/// %IteratorHelperPrototype% - shared prototype for all iterator helpers
/// This is set up to have:
/// - [[Prototype]] = Iterator.prototype
/// - [Symbol.toStringTag] = "Iterator Helper"
/// - next() method that dispatches to the underlying helper
/// - return() method that dispatches to the underlying helper
pub struct IteratorHelperPrototype;

#[props(intrinsic_name = iterator_helper, extends = Iterator)]
impl IteratorHelperPrototype {
    #[nonstatic]
    fn next(#[this] this: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let this_obj = this.to_object()?;
        let helper = this_obj
            .downcast_dyn::<dyn IteratorHelperImpl>()
            .ok_or_else(|| Error::ty("not an iterator helper"))?;
        helper.next_impl(realm)
    }

    #[prop("return")]
    #[nonstatic]
    fn return_method(#[this] this: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let this_obj = this.to_object()?;
        let helper = this_obj
            .downcast_dyn::<dyn IteratorHelperImpl>()
            .ok_or_else(|| Error::ty("not an iterator helper"))?;
        helper.return_impl(realm)
    }

    #[get(Symbol::TO_STRING_TAG)]
    #[nonstatic]
    fn to_string_tag() -> &'static str {
        "Iterator Helper"
    }
}

/// 7.4.15 CreateIterResultObject ( value, done )
pub fn create_iter_result_object(value: Value, done: bool, realm: &mut Realm) -> Res<ObjectHandle> {
    let obj = crate::Object::new(realm);
    obj.define_property("value".into(), value, realm)?;
    obj.define_property("done".into(), done.into(), realm)?;
    Ok(obj)
}

pub fn create_iter_result(value: Value, done: bool, realm: &mut Realm) -> ValueResult {
    Ok(create_iter_result_object(value, done, realm)?.into_value())
}
